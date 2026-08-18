//! Example: Tauri command delegating to UseCase (skeleton).
//! No unwrap/expect — map errors to String or ErrorDto.

// use tauri::State;

#[tauri::command]
pub async fn example_list_items() -> Result<Vec<String>, String> {
    // let uc = ListItems::new(state.repo());
    // uc.execute().map_err(|e| e.to_string())
    Ok(vec![])
}
