use serde::{Deserialize, Serialize};

/// Users new room participation status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomParticipationStatus {
    Joined,
    Left,
}

/// A user has joined or left a room
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomParticipationEvent {
    /// The slug of the room the user has joined or left
    #[serde(rename = "r")]
    pub room: String,
    /// The username of the user that has joined or left
    #[serde(rename = "u")]
    pub username: String,
    /// The new status of the user in the room
    #[serde(rename = "s")]
    pub status: RoomParticipationStatus,
}

/// A user has sent a message to a room
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMessageEvent {
    /// The slug of the room the user has sent the message to
    #[serde(rename = "r")]
    pub room: String,
    /// The username of the user that has sent the message
    #[serde(rename = "u")]
    pub username: String,
    /// The content of the message
    #[serde(rename = "c")]
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
/// Events that can be sent to the client
/// Events maybe related to different users and rooms, the receipient is a single chat session
pub enum Event {
    RoomParticipation(RoomParticipationEvent),
    UserMessage(UserMessageEvent),
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
    fn test_room_participation_join_event() {
        let event = Event::RoomParticipation(RoomParticipationEvent {
            room: "test".to_string(),
            username: "test".to_string(),
            status: RoomParticipationStatus::Joined,
        });

        assert_event_serialization(
            &event,
            r#"{"t":"room_participation","r":"test","u":"test","s":"joined"}"#,
        );
    }

    #[test]
    fn test_room_participation_leave_event() {
        let event = Event::RoomParticipation(RoomParticipationEvent {
            room: "test".to_string(),
            username: "test".to_string(),
            status: RoomParticipationStatus::Left,
        });

        assert_event_serialization(
            &event,
            r#"{"t":"room_participation","r":"test","u":"test","s":"left"}"#,
        );
    }

    #[test]
    fn test_user_message_event() {
        let event = Event::UserMessage(UserMessageEvent {
            room: "test".to_string(),
            username: "test".to_string(),
            content: "test".to_string(),
        });

        assert_event_serialization(
            &event,
            r#"{"t":"user_message","r":"test","u":"test","c":"test"}"#,
        );
    }
}
