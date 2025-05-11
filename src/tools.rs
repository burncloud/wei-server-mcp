use anyhow::Result;
use mcp_core::tool_text_content;
use mcp_core::types::ToolResponseContent;
use mcp_core_macros::tool;

#[tool(
    name = "Add",
    description = "Adds two numbers together.",
    params(a = "The first number to add", b = "The second number to add")
)]
pub async fn add_tool(a: f64, b: f64) -> Result<ToolResponseContent> {
    Ok(tool_text_content!((a + b).to_string()))
}

#[tool(
    name = "Sub",
    description = "Subtract 2nd number from 1st",
    params(a = "The first number", b = "The second number")
)]
pub async fn sub_tool(a: f64, b: f64) -> Result<ToolResponseContent> {
    Ok(tool_text_content!((a -  b).to_string()))
}

#[tool(
    name = "CheckAngel",
    description = "Check if angels exist in this world",
    params(a = "none")
)]
pub async fn check_angel(_a: f64) -> Result<ToolResponseContent> {
    Ok(tool_text_content!("这个世界存在天使！！！".to_string()))
}

#[tool(
    name = "QueryAngelType",
    description = "Query detailed information about different types of angels",
    params(angel_type = "The type of angel to query about")
)]
pub async fn query_angel_type(angel_type: String) -> Result<ToolResponseContent> {
    let description = match angel_type.to_lowercase().as_str() {
        "炽天使" => "炽天使（Seraphim）是最接近上帝的天使，他们环绕在上帝的宝座周围，不断地赞美和歌颂。他们拥有六个翅膀，全身散发着炽热的光芒。他们代表着神圣的爱与光明。",
        "智天使" => "智天使（Cherubim）是守护者和看门人，他们守护着伊甸园和生命树。他们拥有四张面孔和四对翅膀，象征着全知全能。",
        "座天使" => "座天使（Thrones）是公正和权威的象征，他们执行神的公义。他们的形象常被描绘成巨大的轮子，布满了眼睛。",
        "主天使" => "主天使（Dominions）是天堂秩序的监督者，负责管理其他天使的职责。他们手持权杖和宝剑，象征着权威。",
        "力天使" => "力天使（Virtues）负责管理自然界的运行，掌管星辰、行星的运转。他们也负责施行奇迹。",
        "能天使" => "能天使（Powers）是守护天堂秩序的战士，抵抗邪恶势力。他们全副武装，随时准备与黑暗势力战斗。",
        "权天使" => "权天使（Principalities）负责守护国家和大型组织，指导人类的集体活动。他们佩戴皇冠，手持权杖。",
        "大天使" => "大天使（Archangels）是最著名的天使类型，如米迦勒、加百列等。他们是上帝的重要使者，负责传达重要信息。",
        "天使" => "普通天使（Angels）最接近人类，是人类的守护者和保护者。他们传达神的旨意，守护和指引人类。",
        _ => "未知的天使类型。已知的天使类型包括：炽天使、智天使、座天使、主天使、力天使、能天使、权天使、大天使和普通天使。"
    };
    Ok(tool_text_content!(description.to_string()))
}

#[tool(
    name = "QueryGPUSpecs",
    description = "查询NVIDIA H100和A100 GPU的详细配置信息",
    params(gpu_model = "GPU型号，可选值：H100或A100")
)]
pub async fn query_gpu_specs(gpu_model: String) -> Result<ToolResponseContent> {
    let specs = match gpu_model.to_uppercase().as_str() {
        "H100" => "NVIDIA H100 GPU规格：

            - 架构：Hopper架构

            - CUDA核心：14,592个

            - Tensor核心：第四代，456个

            - 性能特点：

              · 相比A100提供高达30倍的推理性能提升

              · GPT-3训练性能提升4倍

              · 新增Transformer引擎，专为大语言模型优化

              · NVLink带宽提升至900 GB/s

            - 主要优势：

              · 适用于下一代AI工作负载

              · 支持FP8精度

              · 增强的MIG功能，支持更灵活的资源分配

              · 配备专用Transformer引擎，可处理万亿参数级语言模型",
        "A100" => "NVIDIA A100 GPU规格：

            - 架构：Ampere架构

            - CUDA核心：6,912个

            - Tensor核心：第三代，432个

            - 性能特点：

              · 相比上一代Volta架构提供20倍性能提升

              · 优秀的通用AI计算性能

              · 支持多实例GPU(MIG)技术

            - 主要优势：

              · 适用于各类高性能计算和AI训练场景

              · 优秀的性价比

              · 成熟稳定的架构

              · 广泛应用于数据中心和云计算",
        _ => "未知的GPU型号。目前支持查询的GPU型号包括：H100和A100。"
    };
    Ok(tool_text_content!(specs.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_core::types::ToolResponseContent;

    // 辅助函数：从ToolResponseContent提取文本内容
    async fn get_text_content(response: Result<ToolResponseContent>) -> String {
        match response {
            Ok(ToolResponseContent::Text { text }) => text,
            _ => "Failed to get text content".to_string(),
        }
    }

    #[tokio::test]
    async fn test_add_tool() {
        // 测试基本加法
        let result = add_tool(5.0, 3.0).await;
        assert_eq!(get_text_content(result).await, "8");

        // 测试负数加法
        let result = add_tool(-5.0, 3.0).await;
        assert_eq!(get_text_content(result).await, "-2");

        // 测试零加法
        let result = add_tool(0.0, 0.0).await;
        assert_eq!(get_text_content(result).await, "0");

        // 测试小数加法
        let result = add_tool(2.5, 3.5).await;
        assert_eq!(get_text_content(result).await, "6");
    }

    #[tokio::test]
    async fn test_sub_tool() {
        // 测试基本减法
        let result = sub_tool(5.0, 3.0).await;
        assert_eq!(get_text_content(result).await, "2");

        // 测试负数减法
        let result = sub_tool(5.0, 8.0).await;
        assert_eq!(get_text_content(result).await, "-3");

        // 测试零减法
        let result = sub_tool(0.0, 0.0).await;
        assert_eq!(get_text_content(result).await, "0");

        // 测试小数减法
        let result = sub_tool(5.5, 2.2).await;
        assert_eq!(get_text_content(result).await, "3.3");
    }

    #[tokio::test]
    async fn test_check_angel() {
        // 测试天使检查工具
        let result = check_angel(0.0).await;
        assert_eq!(get_text_content(result).await, "这个世界存在天使！！！");
    }

    #[tokio::test]
    async fn test_query_angel_type() {
        // 测试已知天使类型
        let types = vec!["炽天使", "智天使", "座天使", "主天使", "力天使", "能天使", "权天使", "大天使", "天使"];
        
        for angel_type in types {
            let result = query_angel_type(angel_type.to_string()).await;
            let content = get_text_content(result).await;
            
            // 验证响应不是默认的未知类型响应
            assert!(content.contains(angel_type));
            assert!(!content.contains("未知的天使类型"));
        }
        
        // 测试未知天使类型
        let result = query_angel_type("未知类型".to_string()).await;
        let content = get_text_content(result).await;
        assert!(content.contains("未知的天使类型"));
    }
    
    #[tokio::test]
    async fn test_query_gpu_specs() {
        // 测试H100 GPU规格
        let result = query_gpu_specs("H100".to_string()).await;
        let content = get_text_content(result).await;
        assert!(content.contains("NVIDIA H100 GPU规格"));
        assert!(content.contains("Hopper架构"));
        
        // 测试大小写不敏感
        let result = query_gpu_specs("h100".to_string()).await;
        let content = get_text_content(result).await;
        assert!(content.contains("NVIDIA H100 GPU规格"));
        
        // 测试A100 GPU规格
        let result = query_gpu_specs("A100".to_string()).await;
        let content = get_text_content(result).await;
        assert!(content.contains("NVIDIA A100 GPU规格"));
        assert!(content.contains("Ampere架构"));
        
        // 测试未知GPU型号
        let result = query_gpu_specs("未知型号".to_string()).await;
        let content = get_text_content(result).await;
        assert!(content.contains("未知的GPU型号"));
    }
}