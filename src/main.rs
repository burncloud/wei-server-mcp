use anyhow::Result;
use mcp_core::types::ClientCapabilities;
use mcp_core::types::Implementation;
use mcp_core::{
    client::ClientBuilder, server::Server, transport::ServerSseTransport, types::ServerCapabilities,
};
use serde_json::json;
use std::fs;
use std::net::TcpListener;
use std::path::Path;

mod tools;
use tools::*;

use rig::{
    completion::Prompt,
    providers::{self},
};

// 从文件读取端口号，如果文件存在
fn read_port_from_file() -> Option<u16> {
    if let Ok(contents) = fs::read_to_string("wei-server-mcp.dat") {
        if let Ok(port) = contents.trim().parse::<u16>() {
            return Some(port);
        }
    }
    None
}

// 保存端口号到文件
fn save_port_to_file(port: u16) -> Result<(), std::io::Error> {
    fs::write("wei-server-mcp.dat", port.to_string())
}

// 检查端口是否可用
fn is_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

// 查找可用端口
fn find_available_port(start_port: u16) -> Option<u16> {
    let mut port = start_port;
    let max_port = 65535;
    
    while port <= max_port {
        if is_port_available(port) {
            return Some(port);
        }
        port += 1;
    }
    None
}

#[tokio::main]
async fn main()->Result<(), anyhow::Error>  {
    tracing_subscriber::fmt::init();
    
    // 初始端口，优先从文件读取，否则默认使用1116
    let initial_port = read_port_from_file().unwrap_or(1116);
    
    // 查找可用端口，从初始端口开始
    let port = match find_available_port(initial_port) {
        Some(p) => p,
        None => {
            eprintln!("无法找到可用端口");
            return Ok(());
        }
    };
    
    // 如果找到的端口与初始端口不同，或者文件不存在，则保存到文件
    if port != initial_port || !Path::new("wei-server-mcp.dat").exists() {
        if let Err(e) = save_port_to_file(port) {
            eprintln!("无法保存端口到文件: {}", e);
        } else {
            println!("端口 {} 已保存到 wei-server-mcp.dat", port);
        }
    }
    
    let mcp_server_protocol = Server::builder("add".to_string(), "1.0".to_string())
        .capabilities(ServerCapabilities {
            tools: Some(json!({
                "listChanged": false,
            })),
            ..Default::default()
        })
        .register_tool(AddTool::tool(), AddTool::call())
        .register_tool(SubTool::tool(), SubTool::call())
        .register_tool(CheckAngel::tool(), CheckAngel::call())
        .register_tool(QueryAngelType::tool(), QueryAngelType::call())
        .register_tool(QueryGpuSpecs::tool(), QueryGpuSpecs::call())
        .build();

    println!("服务器启动于端口: {}", port);
    let mcp_server_transport =
        ServerSseTransport::new("127.0.0.1".to_string(), port, mcp_server_protocol);

    let _ = Server::start(mcp_server_transport.clone()).await;

    // Create the MCP client
    let mcp_client = ClientBuilder::new(mcp_server_transport).build();

    // Start the MCP client
    let _ = mcp_client.open().await;
    let init_res = mcp_client
        .initialize(
            Implementation {
                name: "mcp-client".to_string(),
                version: "0.1.0".to_string(),
            },
            ClientCapabilities::default(),
        )
        .await;
    println!("Initialized: {:?}", init_res);

    let tools_list_res = mcp_client.list_tools(None, None).await;
    println!("Tools: {:?}", tools_list_res);

    tracing::info!("Building RIG agent");
    let completion_model = providers::openai::Client::from_env();
    let mut agent_builder = completion_model.agent("gpt-3.5-turbo-0125");

    // Add MCP tools to the agent
    agent_builder = tools_list_res
        .unwrap()
        .tools
        .into_iter()
        .fold(agent_builder, |builder, tool| {
            builder.mcp_tool(tool, mcp_client.clone().into())
        });
    let agent = agent_builder.build();

    tracing::info!("Prompting RIG agent");
    let response = agent.prompt("Add 10 + 10").await;
    tracing::info!("Agent response: {:?}", response);
    Ok(())
}
