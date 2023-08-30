use std::collections::{HashMap, HashSet};

use super::user_session_handle::UserSessionHandle;

#[derive(Debug)]
pub struct UserRegistry {
    user_id_to_sessions: HashMap<String, HashSet<String>>,
    user_ids: HashSet<String>,
}

/// [UserRegistry] is a smart container for keeping track of which unique list of users are in a room
///
/// Since a user can have multiple sessions, we need to keep track of which sessions belong to which users
impl UserRegistry {
    pub fn new() -> Self {
        UserRegistry {
            user_id_to_sessions: HashMap::new(),
            user_ids: HashSet::new(),
        }
    }

    /// Add a user to the room, returns true if the user is a new user
    pub fn insert(&mut self, user_session_handle: &UserSessionHandle) -> bool {
        let user_id = String::from(user_session_handle.user_id());
        let session_id = String::from(user_session_handle.session_id());

        let sessions = self
            .user_id_to_sessions
            .entry(user_id.clone())
            .or_insert_with(HashSet::new);

        sessions.insert(session_id);

        let is_new_user = sessions.len() == 1;

        if is_new_user {
            self.user_ids.insert(user_id);
        }

        is_new_user
    }

    /// Removes a given session from the participant list, returns true if the user is no longer in the room
    /// Does nothing and returns false if the user does not exist
    pub fn remove(&mut self, user_session_handle: &UserSessionHandle) -> bool {
        let user_id = String::from(user_session_handle.user_id());
        let session_id = String::from(user_session_handle.session_id());

        let to_remove = self.user_id_to_sessions.get_mut(&user_id);

        if let Some(sessions) = to_remove {
            sessions.remove(&session_id);

            if sessions.is_empty() {
                self.user_id_to_sessions.remove(&user_id);
                self.user_ids.remove(&user_id);

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_unique_user_ids(&self) -> Vec<String> {
        self.user_ids.iter().cloned().collect()
    }
}
