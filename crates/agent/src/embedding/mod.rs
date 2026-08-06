//! 本地文本嵌入服务（fastembed + ONNX，纯本地推理）。
//!
//! 用 `EmbeddingModel::BGESmallZHV15`（BAAI/bge-small-zh-v1.5，512 维）为记忆检索
//! 提供向量。模型首次运行联网下载到缓存目录，之后完全离线。
//!
//! 推理在 `spawn_blocking` 中调用（onnx 阻塞），`EmbeddingService` 内部用
//! `OnceLock<Mutex<TextEmbedding>>` 懒加载 + 互斥串行化，`TextEmbedding::embed`
//! 需要 `&mut self`。

use std::fmt;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};

/// bge-small-zh-v1.5 固定输出维度（与 `chat_memory_vec` 的 `vec0(float[512])` 绑定）。
pub const EMBEDDING_DIMS: usize = 512;

/// 本地嵌入服务错误。
#[derive(Debug, Clone)]
pub enum EmbeddingError {
    /// 模型加载失败（首次运行需联网下载，或下载 / 加载出错）。
    Init(String),
    /// 单次推理失败。
    Inference(String),
}

impl fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddingError::Init(message) => write!(f, "embedding model init: {message}"),
            EmbeddingError::Inference(message) => write!(f, "embedding inference: {message}"),
        }
    }
}

impl std::error::Error for EmbeddingError {}

/// 文本嵌入能力（供 RAG 记忆检索注入）。
pub trait Embedder: Send + Sync {
    /// 嵌入维度（与模型绑定，BGE-small-zh-v1.5 = 512）。
    fn dims(&self) -> usize;

    /// 预热底层模型：首次调用会联网下载模型与 onnxruntime 到缓存目录，之后离线可用。
    /// 幂等，可安全重复调用；失败时可稍后由首次 `embed` 惰性重试。
    fn preload(&self) -> Result<(), EmbeddingError>;

    /// 批量嵌入文本，返回与输入一一对应的向量。
    fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// 嵌入单条文本。
    fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.embed_texts(&[text.to_string()])
            .map(|mut vectors| vectors.pop().unwrap_or_default())
    }
}

/// fastembed 实现：懒加载模型，`&self` 可并发安全调用。
pub struct EmbeddingService {
    model: OnceLock<Result<Mutex<TextEmbedding>, EmbeddingError>>,
    cache_dir: Option<PathBuf>,
}

impl EmbeddingService {
    /// 构造服务（不加载模型，首次 `embed` 时懒初始化）。
    ///
    /// # 参数
    /// - `cache_dir` — 模型缓存目录（None 用 fastembed 默认 `.fastembed_cache`）
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        Self {
            model: OnceLock::new(),
            cache_dir,
        }
    }

    fn get_model(&self) -> Result<&Mutex<TextEmbedding>, EmbeddingError> {
        self.model
            .get_or_init(|| {
                let mut options = TextInitOptions::new(EmbeddingModel::BGESmallZHV15)
                    .with_show_download_progress(true);
                if let Some(dir) = &self.cache_dir {
                    options = options.with_cache_dir(dir.clone());
                }
                TextEmbedding::try_new(options)
                    .map(Mutex::new)
                    .map_err(|error| EmbeddingError::Init(error.to_string()))
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}

impl Embedder for EmbeddingService {
    fn dims(&self) -> usize {
        EMBEDDING_DIMS
    }

    fn preload(&self) -> Result<(), EmbeddingError> {
        self.get_model().map(|_| ())
    }

    fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let model = self.get_model()?;
        let mut model = model
            .lock()
            .map_err(|_| EmbeddingError::Inference("embedding model poisoned".into()))?;
        let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
        model
            .embed(refs, None)
            .map_err(|error| EmbeddingError::Inference(error.to_string()))
    }
}
