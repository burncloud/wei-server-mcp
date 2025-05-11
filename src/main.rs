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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;
    use std::net::TcpListener;
    
    // 测试端口可用性检查函数
    #[test]
    fn test_is_port_available() {
        // 查找一个可用端口
        let start_port = 50000;
        let available_port = find_available_port(start_port).unwrap();
        
        // 验证端口可用
        assert!(is_port_available(available_port));
        
        // 占用该端口
        let listener = TcpListener::bind(format!("127.0.0.1:{}", available_port)).unwrap();
        
        // 验证端口不再可用
        assert!(!is_port_available(available_port));
        
        // 释放占用的资源
        drop(listener);
        
        // 端口应该再次可用
        assert!(is_port_available(available_port));
    }
    
    // 测试查找可用端口函数
    #[test]
    fn test_find_available_port() {
        // 查找可用端口
        let start_port = 50100;
        let port1 = find_available_port(start_port).unwrap();
        
        // 端口应该大于等于起始端口
        assert!(port1 >= start_port);
        
        // 占用第一个找到的端口
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port1)).unwrap();
        
        // 再次查找应该找到不同的端口
        let port2 = find_available_port(start_port).unwrap();
        assert_ne!(port1, port2);
        
        // 释放资源
        drop(listener);
    }
    
    // 测试端口保存和读取功能
    #[test]
    fn test_port_file_operations() {
        // 测试用的端口号
        let test_port = 51000;
        let test_file = "wei-server-mcp-test.dat";
        
        // 清理可能存在的测试文件
        let _ = remove_file(test_file);
        
        // 保存端口到文件
        fs::write(test_file, test_port.to_string()).unwrap();
        
        // 读取文件内容
        let contents = fs::read_to_string(test_file).unwrap();
        let read_port = contents.trim().parse::<u16>().unwrap();
        
        // 验证读取的端口与写入的相同
        assert_eq!(test_port, read_port);
        
        // 清理测试文件
        let _ = remove_file(test_file);
    }
    
    // 测试端口自增功能
    #[test]
    fn test_port_increment() {
        // 查找一个可用端口作为起点
        let start_port = 52000;
        let port1 = find_available_port(start_port).unwrap();
        
        // 占用这个端口
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port1)).unwrap();
        
        // 从相同的起始端口开始查找，应该找到下一个可用端口
        let port2 = find_available_port(start_port).unwrap();
        
        // 第二个端口应该大于第一个
        assert!(port2 > port1);
        
        // 释放资源
        drop(listener);
    }
    
    // 测试完整的端口分配流程
    #[test]
    fn test_port_allocation_workflow() {
        // 测试文件名
        let test_file = "wei-server-mcp-test-workflow.dat";
        
        // 清理可能存在的测试文件
        let _ = remove_file(test_file);
        
        // 初始端口
        let initial_port = 53000;
        
        // 写入初始端口到文件
        fs::write(test_file, initial_port.to_string()).unwrap();
        
        // 读取端口
        let contents = fs::read_to_string(test_file).unwrap();
        let read_port = contents.trim().parse::<u16>().unwrap();
        assert_eq!(initial_port, read_port);
        
        // 占用初始端口
        let listener = TcpListener::bind(format!("127.0.0.1:{}", initial_port)).unwrap();
        
        // 模拟主程序中的端口分配逻辑
        let file_port = if let Ok(contents) = fs::read_to_string(test_file) {
            if let Ok(port) = contents.trim().parse::<u16>() {
                port
            } else {
                initial_port
            }
        } else {
            initial_port
        };
        
        // 查找可用端口
        let allocated_port = find_available_port(file_port).unwrap();
        
        // 分配的端口应该大于初始端口（因为初始端口已被占用）
        assert!(allocated_port > initial_port);
        
        // 保存新的端口到文件
        fs::write(test_file, allocated_port.to_string()).unwrap();
        
        // 验证文件已更新
        let new_contents = fs::read_to_string(test_file).unwrap();
        let new_port = new_contents.trim().parse::<u16>().unwrap();
        assert_eq!(allocated_port, new_port);
        
        // 清理资源
        drop(listener);
        let _ = remove_file(test_file);
    }
}
