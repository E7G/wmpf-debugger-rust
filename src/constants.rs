#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CompressAlgo {
    None = 0,
    Zlib = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugMessageCategory {
    SetupContext,
    CallInterface,
    EvaluateJavascript,
    CallInterfaceResult,
    EvaluateJavascriptResult,
    Breakpoint,
    Ping,
    Pong,
    DomOp,
    DomEvent,
    NetworkDebugAPI,
    ChromeDevtools,
    ChromeDevtoolsResult,
    AddJsContext,
    RemoveJsContext,
    ConnectJsContext,
    EngineEvent,
    EngineOp,
    CustomMessage,
}

impl DebugMessageCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "setupContext" => Some(Self::SetupContext),
            "callInterface" => Some(Self::CallInterface),
            "evaluateJavascript" => Some(Self::EvaluateJavascript),
            "callInterfaceResult" => Some(Self::CallInterfaceResult),
            "evaluateJavascriptResult" => Some(Self::EvaluateJavascriptResult),
            "breakpoint" => Some(Self::Breakpoint),
            "ping" => Some(Self::Ping),
            "pong" => Some(Self::Pong),
            "domOp" => Some(Self::DomOp),
            "domEvent" => Some(Self::DomEvent),
            "networkDebugAPI" => Some(Self::NetworkDebugAPI),
            "chromeDevtools" => Some(Self::ChromeDevtools),
            "chromeDevtoolsResult" => Some(Self::ChromeDevtoolsResult),
            "addJsContext" => Some(Self::AddJsContext),
            "removeJsContext" => Some(Self::RemoveJsContext),
            "connectJsContext" => Some(Self::ConnectJsContext),
            "engineEvent" => Some(Self::EngineEvent),
            "engineOp" => Some(Self::EngineOp),
            "customMessage" => Some(Self::CustomMessage),
            _ => None,
        }
    }
}

pub struct Constants;

impl Constants {
    pub fn response_type_name(v: u32) -> &'static str {
        match v {
            2001 => "Heartbeat",
            2002 => "Login",
            3001 => "EventNotifyBegin",
            3002 => "EventNotifyEnd",
            3003 => "EventNotifyBlock",
            2003 => "JoinRoom",
            2000 => "SendDebugMessage",
            2006 => "SendDebugMessageParallelly",
            2004 => "QuitRoom",
            1000 => "MessageNotify",
            1006 => "MessageNotifyParallelly",
            2005 => "SyncMessage",
            _ => "<unknown>",
        }
    }
}
