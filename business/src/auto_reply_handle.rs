use crate::auto_reply::default::InMemoryDefaultReply;
use crate::auto_reply::{
    AutoReplyPipeline, DefaultReplyStore, InMemoryDedup, KeywordFilter, KeywordRule,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AutoReplyHandle {
    pipeline: Arc<AutoReplyPipeline>,
}

impl AutoReplyHandle {
    pub fn new() -> Self {
        let pipeline = AutoReplyPipeline::new(
            KeywordFilter::default(),
            Box::new(InMemoryDedup::default()),
            Vec::<KeywordRule>::new(),
            Box::new(InMemoryDefaultReply::default()),
        );
        Self {
            pipeline: Arc::new(pipeline),
        }
    }

    pub fn pipeline(&self) -> Arc<AutoReplyPipeline> {
        self.pipeline.clone()
    }

    #[allow(dead_code)]
    pub fn with_default_reply(mut self, default_reply: Box<dyn DefaultReplyStore>) -> Self {
        let pipeline = AutoReplyPipeline::new(
            KeywordFilter::default(),
            Box::new(InMemoryDedup::default()),
            Vec::<KeywordRule>::new(),
            default_reply,
        );
        self.pipeline = Arc::new(pipeline);
        self
    }
}

impl Default for AutoReplyHandle {
    fn default() -> Self {
        Self::new()
    }
}
