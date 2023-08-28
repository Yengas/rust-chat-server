use serde::{Deserialize, Serialize};

/// User Command for joining a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JoinRoomCommand {
    // The room to join.
    #[serde(rename = "r")]
    pub room: String,
}

/// User Command for leaving a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveRoomCommand {
    // The room to leave.
    #[serde(rename = "r")]
    pub room: String,
}

/// User Command for sending a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendMessageCommand {
    // The room to send the message to.
    #[serde(rename = "r")]
    pub room: String,
    // The content of the message.
    #[serde(rename = "c")]
    pub content: String,
}

/// User Command for quitting the whole chat session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuitCommand;

/// A user command which can be sent to the server by a single user session.
/// All commands are processed in the context of the chat server paired with an individual user session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "_ct", rename_all = "snake_case")]
pub enum UserCommand {
    JoinRoom(JoinRoomCommand),
    LeaveRoom(LeaveRoomCommand),
    SendMessage(SendMessageCommand),
    Quit(QuitCommand),
}

#[cfg(test)]
mod tests {
    use super::*;

    // given a command enum, and an expect string, asserts that command is serialized / deserialized appropiately
    fn assert_command_serialization(command: &UserCommand, expected: &str) {
        let serialized = serde_json::to_string(&command).unwrap();
        assert_eq!(serialized, expected);
        let deserialized: UserCommand = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, *command);
    }

    #[test]
    fn test_join_command() {
        let command = UserCommand::JoinRoom(JoinRoomCommand {
            room: "test".to_string(),
        });

        assert_command_serialization(&command, r#"{"_ct":"join_room","r":"test"}"#);
    }

    #[test]
    fn test_leave_command() {
        let command = UserCommand::LeaveRoom(LeaveRoomCommand {
            room: "test".to_string(),
        });

        assert_command_serialization(&command, r#"{"_ct":"leave_room","r":"test"}"#);
    }

    #[test]
    fn test_message_command() {
        let command = UserCommand::SendMessage(SendMessageCommand {
            room: "test".to_string(),
            content: "test".to_string(),
        });

        assert_command_serialization(&command, r#"{"_ct":"send_message","r":"test","c":"test"}"#);
    }

    #[test]
    fn test_quit_command() {
        let command = UserCommand::Quit(QuitCommand);

        assert_command_serialization(&command, r#"{"_ct":"quit"}"#);
    }
}
