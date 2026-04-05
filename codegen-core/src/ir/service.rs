//! Service definitions for gRPC services.

/// Definition of a gRPC service.
#[derive(Debug, Clone)]
pub struct ServiceDef {
    /// The service name (e.g., `"Greeter"`).
    pub name: String,
    /// The fully-qualified package (e.g., `"helloworld"`).
    pub package: String,
    /// The proto identifier (e.g., `"Greeter"`).
    pub proto_name: String,
    /// RPC methods in this service.
    pub methods: Vec<MethodDef>,
    /// Doc comments.
    pub comments: Vec<String>,
}

impl ServiceDef {
    /// Fully-qualified gRPC service name (e.g., `"helloworld.Greeter"`).
    pub fn fully_qualified_name(&self) -> String {
        if self.package.is_empty() {
            self.proto_name.clone()
        } else {
            format!("{}.{}", self.package, self.proto_name)
        }
    }

    /// gRPC path for this method (e.g., `"/helloworld.Greeter/SayHello"`).
    pub fn method_grpc_path(&self, method: &MethodDef) -> String {
        debug_assert!(
            !method.proto_name.contains('/'),
            "proto_name must not contain '/', got: {:?}",
            method.proto_name
        );
        format!("/{}/{}", self.fully_qualified_name(), method.proto_name)
    }

    /// Validate that this definition will produce correct generated code.
    ///
    /// Returns a list of problems. An empty vec means the definition is valid.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("service name is empty".into());
        }
        if self.proto_name.is_empty() {
            errors.push("service proto_name is empty".into());
        }
        for (i, m) in self.methods.iter().enumerate() {
            if m.name.is_empty() {
                errors.push(format!("method[{i}] name is empty"));
            }
            if m.proto_name.is_empty() {
                errors.push(format!("method[{i}] proto_name is empty"));
            }
            if m.proto_name.contains('/') {
                errors.push(format!(
                    "method[{i}] proto_name `{}` contains '/'",
                    m.proto_name
                ));
            }
            if m.input_type.is_empty() {
                errors.push(format!("method[{i}] `{}` input_type is empty", m.name));
            }
            if m.output_type.is_empty() {
                errors.push(format!("method[{i}] `{}` output_type is empty", m.name));
            }
        }
        errors
    }
}

/// Definition of a single RPC method.
#[derive(Debug, Clone)]
pub struct MethodDef {
    /// Rust-style method name in snake_case (e.g., `"say_hello"`).
    pub name: String,
    /// Proto-style method name (e.g., `"SayHello"`).
    pub proto_name: String,
    /// Fully-qualified input type path (e.g., `"crate::HelloRequest"`).
    pub input_type: String,
    /// Fully-qualified output type path (e.g., `"crate::HelloReply"`).
    pub output_type: String,
    /// Whether the client sends a stream of messages.
    pub client_streaming: bool,
    /// Whether the server returns a stream of messages.
    pub server_streaming: bool,
    /// The codec path to use (e.g., `"crate::codec::Codec"`).
    pub codec_path: String,
    /// Doc comments.
    pub comments: Vec<String>,
}

impl MethodDef {
    /// Streaming mode as a human-readable string.
    pub fn streaming_mode(&self) -> &'static str {
        match (self.client_streaming, self.server_streaming) {
            (false, false) => "unary",
            (false, true) => "server streaming",
            (true, false) => "client streaming",
            (true, true) => "bidi streaming",
        }
    }
}
