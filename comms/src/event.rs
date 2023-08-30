use serde::{Deserialize, Serialize};

/// The detail of a given room
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomDetail {
    /// The slug of the room
    #[serde(rename = "n")]
    pub name: String,
    /// The description of the room
    #[serde(rename = "d")]
    pub description: String,
}

/// A user has successfully logged in
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginSuccessfulReplyEvent {
    /// The session id for the connection
    #[serde(rename = "s")]
    pub session_id: String,
    /// The id of the user that has logged in
    #[serde(rename = "u")]
    pub user_id: String,
    /// The list of rooms the user can participate, unique and ordered
    #[serde(rename = "rs")]
    pub rooms: Vec<RoomDetail>,
}

/// Users new room participation status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomParticipationStatus {
    Joined,
    Left,
}

/// A user has joined or left a room
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomParticipationBroacastEvent {
    /// The slug of the room the user has joined or left
    #[serde(rename = "r")]
    pub room: String,
    /// The id of the user that has joined or left
    #[serde(rename = "u")]
    pub user_id: String,
    /// The new status of the user in the room
    #[serde(rename = "s")]
    pub status: RoomParticipationStatus,
}

/// A reply to the user when they have joined a room
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserJoinedRoomReplyEvent {
    /// The slug of the room the user has joined
    #[serde(rename = "r")]
    pub room: String,
    /// The users currently in the room, unique and ordered
    #[serde(rename = "us")]
    pub users: Vec<String>,
}

/// A user has sent a message to a room
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMessageBroadcastEvent {
    /// The slug of the room the user has sent the message to
    #[serde(rename = "r")]
    pub room: String,
    /// The id of the user that has sent the message
    #[serde(rename = "u")]
    pub user_id: String,
    /// The content of the message
    #[serde(rename = "c")]
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "_et", rename_all = "snake_case")]
/// Events that can be sent to the client
/// Events maybe related to different users and rooms, the receipient is a single chat session
pub enum Event {
    LoginSuccessful(LoginSuccessfulReplyEvent),
    RoomParticipation(RoomParticipationBroacastEvent),
    UserJoinedRoom(UserJoinedRoomReplyEvent),
    UserMessage(UserMessageBroadcastEvent),
}

#[cfg(test)]
mod tests {
    use super::*;

    // given an event enum, and an expect string, asserts that event is serialized / deserialized appropiately
    fn assert_event_serialization(event: &Event, expected: &str) {
        let serialized = serde_json::to_string(&event).unwrap();
        assert_eq!(serialized, expected);
        let deserialized: Event = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, *event);
    }

    #[test]
    fn test_login_successful_event() {
        let event = Event::LoginSuccessful(LoginSuccessfulReplyEvent {
            session_id: "session-id-1".to_string(),
            user_id: "user-id-1".to_string(),
            rooms: vec![RoomDetail {
                name: "room-1".to_string(),
                description: "some description".to_string(),
            }],
        });

        assert_event_serialization(
            &event,
            r#"{"_et":"login_successful","s":"session-id-1","u":"user-id-1","rs":[{"n":"room-1","d":"some description"}]}"#,
        );
    }

    #[test]
    fn test_room_participation_join_event() {
        let event = Event::RoomParticipation(RoomParticipationBroacastEvent {
            room: "test".to_string(),
            user_id: "test".to_string(),
            status: RoomParticipationStatus::Joined,
        });

        assert_event_serialization(
            &event,
            r#"{"_et":"room_participation","r":"test","u":"test","s":"joined"}"#,
        );
    }

    #[test]
    fn test_room_participation_leave_event() {
        let event = Event::RoomParticipation(RoomParticipationBroacastEvent {
            room: "test".to_string(),
            user_id: "test".to_string(),
            status: RoomParticipationStatus::Left,
        });

        assert_event_serialization(
            &event,
            r#"{"_et":"room_participation","r":"test","u":"test","s":"left"}"#,
        );
    }

    #[test]
    fn test_user_joined_room_event() {
        let event = Event::UserJoinedRoom(UserJoinedRoomReplyEvent {
            room: "test".to_string(),
            users: vec!["test".to_string()],
        });

        assert_event_serialization(
            &event,
            r#"{"_et":"user_joined_room","r":"test","us":["test"]}"#,
        );
    }

    #[test]
    fn test_user_message_event() {
        let event = Event::UserMessage(UserMessageBroadcastEvent {
            room: "test".to_string(),
            user_id: "test".to_string(),
            content: "test".to_string(),
        });

        assert_event_serialization(
            &event,
            r#"{"_et":"user_message","r":"test","u":"test","c":"test"}"#,
        );
    }
}
