use super::BuildChatPrompt;
use crate::error::Result;
use endpoints::chat::{ChatCompletionRequestMessage, ChatCompletionRole};

/// Generate prompts for the `Mistral-instruct-v0.1` model.
#[derive(Debug, Default, Clone)]
pub struct MistralInstructPrompt;
impl MistralInstructPrompt {
    /// Create a system prompt from a chat completion request message.
    fn create_system_prompt(&self, system_message: &ChatCompletionRequestMessage) -> String {
        format!(
            "<<SYS>>\n{content} <</SYS>>",
            content = system_message.content.as_str()
        )
    }

    /// Create a user prompt from a chat completion request message.
    fn append_user_message(
        &self,
        chat_history: impl AsRef<str>,
        system_prompt: impl AsRef<str>,
        content: impl AsRef<str>,
    ) -> String {
        match chat_history.as_ref().is_empty() {
            true => match system_prompt.as_ref().is_empty() {
                true => {
                    format!(
                        "<s>[INST] {user_message} [/INST]",
                        user_message = content.as_ref().trim(),
                    )
                }
                false => {
                    format!(
                        "<s>[INST] {system_prompt}\n\n{user_message} [/INST]",
                        system_prompt = system_prompt.as_ref().trim(),
                        user_message = content.as_ref().trim(),
                    )
                }
            },
            false => format!(
                "{chat_history}<s>[INST] {user_message} [/INST]",
                chat_history = chat_history.as_ref().trim(),
                user_message = content.as_ref().trim(),
            ),
        }
    }

    /// create an assistant prompt from a chat completion request message.
    fn append_assistant_message(
        &self,
        chat_history: impl AsRef<str>,
        content: impl AsRef<str>,
    ) -> String {
        format!(
            "{prompt} {assistant_message} </s>",
            prompt = chat_history.as_ref().trim(),
            assistant_message = content.as_ref().trim(),
        )
    }
}
impl BuildChatPrompt for MistralInstructPrompt {
    fn build(&self, messages: &mut Vec<ChatCompletionRequestMessage>) -> Result<String> {
        if messages.is_empty() {
            return Ok(String::new());
        }

        // systemp prompt
        let system_prompt = if messages[0].role == ChatCompletionRole::System {
            let system_message = messages.remove(0);
            self.create_system_prompt(&system_message)
        } else {
            String::from("<<SYS>>\nYou are a helpful, respectful and honest assistant. Always answer as short as possible, while being safe. <</SYS>>")
        };

        // append user/assistant messages
        if messages.is_empty() {
            return Err(crate::error::PromptError::NoMessages);
        }

        let mut prompt = String::new();
        for message in messages {
            if message.role == ChatCompletionRole::User {
                prompt =
                    self.append_user_message(&prompt, &system_prompt, message.content.as_str());
            } else if message.role == ChatCompletionRole::Assistant {
                prompt = self.append_assistant_message(&prompt, message.content.as_str());
            } else {
                return Err(crate::error::PromptError::UnknownRole(message.role));
            }
        }

        // println!("*** [prompt begin] ***");
        // println!("{}", &prompt);
        // println!("*** [prompt end] ***");

        Ok(prompt)
    }
}