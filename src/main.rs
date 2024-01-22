use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        ChatCompletionResponseFormat,
        ChatCompletionResponseFormatType,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::{ fs::File };
use std::io::Write;
use gen_pair_local::{ convert_to_text_vec };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_file = "src/k8s.md";

    let converted = convert_to_text_vec(input_file)?;
    let json_output = serde_json::to_string_pretty(&converted).unwrap();

    let mut file = File::create("segmented_text.json").expect(
        "Error creating file `segmented_text.json`"
    );
    file.write_all(json_output.as_bytes()).expect("Error writing to file `segmented_text.json`");

    Ok(())
}
pub async fn gen_pair(
    input_vec: Vec<String>
) -> Result<Option<Vec<(String, String)>>, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct QaPair {
        question: String,
        answer: String,
    }
    let sys_prompt = env
        ::var("SYS_PROMPT")
        .unwrap_or(
            "As a highly skilled assistant, you are tasked with generating informative question and answer pairs from the provided text. Focus on crafting Q&A pairs that are relevant to the primary subject matter of the text. Your questions should be engaging and answers concise, avoiding details of specific examples that are not representative of the text's broader themes. Aim for a comprehensive understanding that captures the essence of the content without being sidetracked by less relevant details.".into()
        );

    let mut qa_pairs_vec = Vec::new();
    let client = Client::new();

    for msg in &input_vec {
        let head = msg.chars().take(20).collect::<String>();
        println!("Processing: {}...", head);
        let user_input =
            format!("
        Here is the user input to work with:
        ---
        {}
        ---
        Your task is to dissect this text for its central themes and most significant details, crafting question and answer pairs that reflect the core message and primary content. Avoid questions about specific examples that do not contribute to the overall understanding of the subject. The questions should cover different types: factual, inferential, thematic, etc., and answers must be concise and pertinent to the text's main intent. Please generate as many relevant question and answers as possible, focusing on the significance and relevance of each to the text's main topic. Provide the results in the following JSON format:
        {{
            \"qa_pairs\": [
                {{
                    \"question\": \"<Your question>\",
                    \"answer\": \"<Your answer>\"
                }},
                // ... additional Q&A pairs based on text relevance
            ]
        }}", msg);
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(&sys_prompt)
                .build()
                .expect("Failed to build system message")
                .into(),
            ChatCompletionRequestUserMessageArgs::default().content(user_input).build()?.into()
        ];

        let response_format = ChatCompletionResponseFormat {
            r#type: ChatCompletionResponseFormatType::JsonObject,
        };

        let request = CreateChatCompletionRequestArgs::default()
            // .max_tokens(6000u16)
            .model("gpt-3.5-turbo-1106")
            .messages(messages)
            .response_format(response_format)
            .build()?;

        let chat = match client.chat().create(request).await {
            Ok(chat) => chat,
            Err(err) => {
                eprintln!("Failed to create chat: {:?}", err);
                continue; // Skip this message and continue with the next one
            }
        };

        if let Some(qa_pairs_json) = &chat.choices[0].message.content {
            match serde_json::from_str::<HashMap<String, Vec<QaPair>>>(qa_pairs_json) {
                Ok(deserialized) => {
                    if let Some(qa_pairs) = deserialized.get("qa_pairs") {
                        let pairs = qa_pairs
                            .iter()
                            .map(|qa| (qa.question.clone(), qa.answer.clone()))
                            .collect::<Vec<(String, String)>>();
                        qa_pairs_vec.extend(pairs);
                    }
                }
                Err(err) => {
                    eprintln!("Failed to deserialize response JSON: {}", err);
                }
            }
        }
    }
    let json_value = json!(
        qa_pairs_vec
            .iter()
            .map(|(question, answer)| json!({question: answer}))
            .collect::<Vec<_>>()
    );

    let json_string = serde_json::to_string_pretty(&json_value)?;

    fs::write("generated_qa.json", json_string)?;

    Ok(Some(qa_pairs_vec))
}
