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
use pandoc_types::definition::*;
use gen_pair_local::Document;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_contents = include_str!("../segmented_text.json");

    let raw_md = include_str!("../k8s.json");

    let pandoc_data: Pandoc = serde_json::from_str(raw_md).expect("failed to parse json");
    let mut doc = Document::new();
    for block in &pandoc_data.blocks {
        doc.action(&block);
    }

    doc.finalize();
    println!("{:#?}", pandoc_data);

    return Ok(());
    let data: Vec<String> = serde_json::from_str(json_contents).expect("failed to parse json");
    let mut count = 0;
    if let Ok(Some(qa_pairs)) = gen_pair(data).await {
        for (question, answer) in qa_pairs {
            count += 1;
            println!(
                "{} Q: {} \t A: {}\n",
                count,
                question.chars().take(30).collect::<String>(),
                answer.chars().take(30).collect::<String>()
            );
        }
    }

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
