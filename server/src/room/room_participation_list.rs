use std::collections::{HashMap, HashSet};

use super::MessageSender;

#[derive(Debug)]
pub struct RoomParticipationList {
    username_to_sessions: HashMap<String, HashSet<String>>,
    usernames: HashSet<String>,
}

impl RoomParticipationList {
    pub fn new() -> Self {
        RoomParticipationList {
            username_to_sessions: HashMap::new(),
            usernames: HashSet::new(),
        }
    }

    /// Add a user to the room, returns true if the user is a new user
    pub fn insert_user(&mut self, message_sender: &MessageSender) -> bool {
        let username = String::from(message_sender.username());
        let session_id = String::from(message_sender.session_id());

        let sessions = self
            .username_to_sessions
            .entry(username.clone())
            .or_insert_with(HashSet::new);

        sessions.insert(session_id);

        let is_new_user = sessions.len() == 1;

        if is_new_user {
            self.usernames.insert(username);
        }

        is_new_user
    }

    /// Removes a given session from the participant list, returns true if the user is no longer in the room
    /// Does nothing and returns false if the user does not exist
    pub fn remove_user_by_session(&mut self, message_sender: &MessageSender) -> bool {
        let username = String::from(message_sender.username());
        let session_id = String::from(message_sender.session_id());

        let to_remove = self.username_to_sessions.get_mut(&username);

        if let Some(sessions) = to_remove {
            sessions.remove(&session_id);

            if sessions.is_empty() {
                self.username_to_sessions.remove(&username);
                self.usernames.remove(&username);

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_unique_usernames(&self) -> Vec<String> {
        self.usernames.iter().cloned().collect()
    }
}
