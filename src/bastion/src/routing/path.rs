//!
//! Module with structs for handling paths on the cluster, a system
//! or a local group level.
//!
use std::net::SocketAddr;
use std::string::ToString;
use uuid::Uuid;

/// Special wrapper for handling actor's path and
/// message distribution.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ActorPath {
    // Node name in the cluster.
    node_name: String,
    // Defines actors in the local or the remote node.
    node_type: ActorNodeType,
    // Defines actors in the top-level namespace.
    scope: ActorScope,
    // A unique identifier of the actor.
    id: String,
}

/// A part of path that defines remote or local machine
/// with running supervisors and actors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ActorNodeType {
    /// The message must be delivered in terms of
    /// the local node.
    Local,
    /// The message must be delivered to the remote
    /// node in the cluster by the certain host and port.
    Remote(SocketAddr),
}

/// A part of path that defines to what part of the node
/// the message must be delivered.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ActorScope {
    /// Broadcast the message to user-defined actors, defined
    /// before starting an application.
    User,
    /// Broadcast the message to top-level built-in actors. For
    /// example it can be logging, configuration, heartbeat actors.
    System,
    /// The message wasn't delivered because the node was
    /// stopped or not available.
    DeadLetter,
    /// The message must be delivered to short-living actors or subtrees of
    /// actors spawned in runtime.
    Temporary,
}

impl ActorPath {
    /// Returns a ActorPath instance, constructed from parts.
    pub(crate) fn new(
        node_name: &str,
        node_type: ActorNodeType,
        scope: ActorScope,
        id: &str,
    ) -> Self {
        ActorPath {
            node_name: node_name.to_string(),
            node_type,
            scope,
            id: id.to_string(),
        }
    }

    /// Replaces the existing node name onto the new one.
    pub fn node_name(mut self, node_name: &str) -> Self {
        self.node_name = node_name.to_string();
        self
    }

    /// Replaces the existing node type onto the new one.
    pub fn node_type(mut self, node_type: ActorNodeType) -> Self {
        self.node_type = node_type;
        self
    }

    /// Replaces the existing scope onto the new one.
    pub fn scope(mut self, scope: ActorScope) -> Self {
        self.scope = scope;
        self
    }

    /// Replaces the existing actor name onto the new one.
    pub fn name(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }
}

impl Default for ActorPath {
    fn default() -> Self {
        let unique_id = Uuid::new_v4().to_string();
        ActorPath::new("node", ActorNodeType::Local, ActorScope::System, &unique_id)
    }
}

impl ToString for ActorPath {
    fn to_string(&self) -> String {
        let node_type = self.node_type.to_string();
        let scope = self.scope.as_str();
        format!(
            "bastion://{}{}/{}/{}",
            self.node_name, node_type, scope, self.id
        )
    }
}

impl ToString for ActorNodeType {
    fn to_string(&self) -> String {
        match self {
            ActorNodeType::Local => String::new(),
            ActorNodeType::Remote(address) => format!("@{}", address.to_string()),
        }
    }
}

impl ActorScope {
    fn as_str(&self) -> &str {
        match self {
            ActorScope::User => "user",
            ActorScope::System => "system",
            ActorScope::DeadLetter => "dead_letter",
            ActorScope::Temporary => "temporary",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::routing::path::{ActorNodeType, ActorPath, ActorScope};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn construct_local_user_path_group() {
        let instance = ActorPath::default()
            .node_name("test")
            .node_type(ActorNodeType::Local)
            .scope(ActorScope::User)
            .name("processing/1");

        assert_eq!(instance.to_string(), "bastion://test/user/processing/1");
    }

    #[test]
    fn construct_local_system_path_group() {
        let instance = ActorPath::default()
            .node_name("test")
            .node_type(ActorNodeType::Local)
            .scope(ActorScope::System)
            .name("processing/1");

        assert_eq!(instance.to_string(), "bastion://test/system/processing/1");
    }

    #[test]
    fn construct_local_deadletter_path_group() {
        let instance = ActorPath::default()
            .node_name("test")
            .node_type(ActorNodeType::Local)
            .scope(ActorScope::DeadLetter)
            .name("processing/1");

        assert_eq!(
            instance.to_string(),
            "bastion://test/dead_letter/processing/1"
        );
    }

    #[test]
    fn construct_local_temporary_path_group() {
        let instance = ActorPath::default()
            .node_name("test")
            .node_type(ActorNodeType::Local)
            .scope(ActorScope::Temporary)
            .name("processing/1");

        assert_eq!(
            instance.to_string(),
            "bastion://test/temporary/processing/1"
        );
    }

    #[test]
    fn construct_remote_user_path_group() {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let instance = ActorPath::default()
            .node_name("test")
            .node_type(ActorNodeType::Remote(address))
            .scope(ActorScope::Temporary)
            .name("processing/1");

        assert_eq!(
            instance.to_string(),
            "bastion://test@127.0.0.1:8080/temporary/processing/1"
        );
    }

    #[test]
    fn construct_remote_system_path_group() {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let instance = ActorPath::default()
            .node_name("test")
            .node_type(ActorNodeType::Remote(address))
            .scope(ActorScope::System)
            .name("processing/1");

        assert_eq!(
            instance.to_string(),
            "bastion://test@127.0.0.1:8080/system/processing/1"
        );
    }

    #[test]
    fn construct_remote_deadletter_path_group() {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let instance = ActorPath::default()
            .node_name("test")
            .node_type(ActorNodeType::Remote(address))
            .scope(ActorScope::DeadLetter)
            .name("processing/1");

        assert_eq!(
            instance.to_string(),
            "bastion://test@127.0.0.1:8080/dead_letter/processing/1"
        );
    }

    #[test]
    fn construct_remote_temporary_path_group() {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let instance = ActorPath::default()
            .node_name("test")
            .node_type(ActorNodeType::Remote(address))
            .scope(ActorScope::Temporary)
            .name("processing/1");

        assert_eq!(
            instance.to_string(),
            "bastion://test@127.0.0.1:8080/temporary/processing/1"
        );
    }

    #[test]
    fn construct_local_user_path_without_group() {
        let instance = ActorPath::default()
            .node_type(ActorNodeType::Local)
            .scope(ActorScope::User)
            .name("1");

        assert_eq!(instance.to_string(), "bastion://node/user/1");
    }

    #[test]
    fn construct_local_system_path_without_group() {
        let instance = ActorPath::default()
            .node_type(ActorNodeType::Local)
            .scope(ActorScope::System)
            .name("1");

        assert_eq!(instance.to_string(), "bastion://node/system/1");
    }

    #[test]
    fn construct_local_deadletter_path_without_group() {
        let instance = ActorPath::default()
            .node_type(ActorNodeType::Local)
            .scope(ActorScope::DeadLetter)
            .name("1");

        assert_eq!(instance.to_string(), "bastion://node/dead_letter/1");
    }

    #[test]
    fn construct_local_temporary_path_without_group() {
        let instance = ActorPath::default()
            .node_type(ActorNodeType::Local)
            .scope(ActorScope::Temporary)
            .name("1");

        assert_eq!(instance.to_string(), "bastion://node/temporary/1");
    }

    #[test]
    fn construct_remote_user_path_without_group() {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let instance = ActorPath::default()
            .node_type(ActorNodeType::Remote(address))
            .scope(ActorScope::User)
            .name("1");

        assert_eq!(instance.to_string(), "bastion://node@127.0.0.1:8080/user/1");
    }

    #[test]
    fn construct_remote_system_path_without_group() {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let instance = ActorPath::default()
            .node_type(ActorNodeType::Remote(address))
            .scope(ActorScope::System)
            .name("1");

        assert_eq!(
            instance.to_string(),
            "bastion://node@127.0.0.1:8080/system/1"
        );
    }

    #[test]
    fn construct_remote_deadletter_path_without_group() {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let instance = ActorPath::default()
            .node_type(ActorNodeType::Remote(address))
            .scope(ActorScope::DeadLetter)
            .name("1");

        assert_eq!(
            instance.to_string(),
            "bastion://node@127.0.0.1:8080/dead_letter/1"
        );
    }

    #[test]
    fn construct_remote_temporary_path_without_group() {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let instance = ActorPath::default()
            .node_type(ActorNodeType::Remote(address))
            .scope(ActorScope::Temporary)
            .name("1");

        assert_eq!(
            instance.to_string(),
            "bastion://node@127.0.0.1:8080/temporary/1"
        );
    }
}
