//! File browser navigation state.

use serde::{Deserialize, Serialize};

/// Field to sort directory listings by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortField {
    Name,
    Size,
    Modified,
}

/// Navigation state for a file browser panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBrowserState {
    pub current_path: String,
    pub history: Vec<String>,
    pub show_hidden: bool,
    pub sort_by: SortField,
}

impl FileBrowserState {
    pub fn new(initial_path: String) -> Self {
        Self {
            current_path: initial_path,
            history: Vec::new(),
            show_hidden: false,
            sort_by: SortField::Name,
        }
    }

    /// Navigate to a new path, pushing the current path to history.
    pub fn navigate(&mut self, path: String) {
        self.history.push(self.current_path.clone());
        self.current_path = path;
    }

    /// Go back to the previous path in history.
    pub fn go_back(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.current_path = prev;
            true
        } else {
            false
        }
    }

    /// Navigate to the parent directory.
    pub fn go_up(&mut self) -> bool {
        let parent = if self.current_path == "/" {
            return false;
        } else {
            let trimmed = self.current_path.trim_end_matches('/');
            match trimmed.rfind('/') {
                Some(0) => "/".to_string(),
                Some(idx) => trimmed[..idx].to_string(),
                None => return false,
            }
        };
        self.navigate(parent);
        true
    }

    /// Toggle hidden files visibility.
    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
    }
}

impl Default for FileBrowserState {
    fn default() -> Self {
        Self::new("/".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_browser_state() {
        let state = FileBrowserState::new("/home/user".into());
        assert_eq!(state.current_path, "/home/user");
        assert!(state.history.is_empty());
        assert!(!state.show_hidden);
        assert_eq!(state.sort_by, SortField::Name);
    }

    #[test]
    fn test_navigate_and_go_back() {
        let mut state = FileBrowserState::new("/home".into());
        state.navigate("/home/user".into());
        assert_eq!(state.current_path, "/home/user");
        assert_eq!(state.history, vec!["/home"]);

        state.navigate("/home/user/docs".into());
        assert_eq!(state.current_path, "/home/user/docs");
        assert_eq!(state.history, vec!["/home", "/home/user"]);

        assert!(state.go_back());
        assert_eq!(state.current_path, "/home/user");
        assert!(state.go_back());
        assert_eq!(state.current_path, "/home");
        assert!(!state.go_back());
    }

    #[test]
    fn test_go_up() {
        let mut state = FileBrowserState::new("/home/user/docs".into());
        assert!(state.go_up());
        assert_eq!(state.current_path, "/home/user");
        assert!(state.go_up());
        assert_eq!(state.current_path, "/home");
        assert!(state.go_up());
        assert_eq!(state.current_path, "/");
        assert!(!state.go_up());
    }

    #[test]
    fn test_toggle_hidden() {
        let mut state = FileBrowserState::default();
        assert!(!state.show_hidden);
        state.toggle_hidden();
        assert!(state.show_hidden);
        state.toggle_hidden();
        assert!(!state.show_hidden);
    }

    #[test]
    fn test_sort_field_serde() {
        let fields = vec![
            (SortField::Name, "\"name\""),
            (SortField::Size, "\"size\""),
            (SortField::Modified, "\"modified\""),
        ];
        for (field, expected) in fields {
            assert_eq!(serde_json::to_string(&field).unwrap(), expected);
            let deserialized: SortField = serde_json::from_str(expected).unwrap();
            assert_eq!(deserialized, field);
        }
    }

    #[test]
    fn test_browser_state_serde_roundtrip() {
        let mut state = FileBrowserState::new("/home/user".into());
        state.navigate("/home/user/docs".into());
        state.show_hidden = true;
        state.sort_by = SortField::Modified;

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: FileBrowserState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.current_path, "/home/user/docs");
        assert_eq!(deserialized.history, vec!["/home/user"]);
        assert!(deserialized.show_hidden);
        assert_eq!(deserialized.sort_by, SortField::Modified);
    }

    #[test]
    fn test_default() {
        let state = FileBrowserState::default();
        assert_eq!(state.current_path, "/");
        assert!(state.history.is_empty());
    }
}
