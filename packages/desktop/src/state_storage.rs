//! localStorage-based state storage for desktop platform

use dioxus::prelude::*;
use serde_json::Value;

/// Save application state to localStorage (requires eval from component)
pub async fn save_state_to_local_storage(eval: &Eval, json: &str) -> Result<(), String> {
    let json_escaped = serde_json::to_string(json).map_err(|e| e.to_string())?;
    
    let js_code = format!(
        r#"
        (function() {{
            try {{
                localStorage.setItem("wco-player-state", {});
                return JSON.stringify({{ success: true }});
            }} catch (e) {{
                return JSON.stringify({{ success: false, error: e.message }});
            }}
        }})()
        "#,
        json_escaped
    );
    
    let result_str = eval(&js_code).await.map_err(|e| e.to_string())?;
    let result: Value = serde_json::from_str(&result_str).map_err(|e| e.to_string())?;
    
    if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
        Ok(())
    } else {
        let error = result
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        Err(error.to_string())
    }
}

/// Load application state from localStorage (requires eval from component)
pub async fn load_state_from_local_storage(eval: &Eval) -> Result<Option<String>, String> {
    let js_code = r#"
        (function() {
            try {
                const value = localStorage.getItem("wco-player-state");
                return value ? JSON.stringify(value) : JSON.stringify(null);
            } catch (e) {
                return JSON.stringify(null);
            }
        })()
    "#;
    
    let result_str = eval(js_code).await.map_err(|e| e.to_string())?;
    
    if result_str == "null" || result_str.is_empty() {
        return Ok(None);
    }
    
    // Parse the JSON string that was returned
    let value: String = serde_json::from_str(&result_str).map_err(|e| e.to_string())?;
    Ok(Some(value))
}
