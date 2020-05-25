use actix::prelude::*;
use actix_raft::{AppData, AppDataResponse, AppError, RaftStorage};
use actix_raft::messages as raft_protocol;
use tracing::*;
use crate::network::node::{Node, LocalNode, RemoteNode};
use crate::raft::Raft;
use crate::ports::http::entities as port_entities;
use serde::{Serialize, Deserialize};

pub trait RaftProtocolBehavior<D: AppData> {
    fn append_entries(
        &self,
        rpc: raft_protocol::AppendEntriesRequest<D>,
        ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = raft_protocol::AppendEntriesResponse, Error = ()>>;
    fn install_snapshot(
        &self,
        rpc: raft_protocol::InstallSnapshotRequest,
        ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = raft_protocol::InstallSnapshotResponse, Error = ()>>;

    fn vote(
        &self,
        rpc: raft_protocol::VoteRequest,
        ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = raft_protocol::VoteResponse, Error = ()>>;
}


impl<D, R, E, S> RaftProtocolBehavior<D> for LocalNode<D, R, E, S>
    where
        D: AppData,
        R: AppDataResponse,
        E: AppError,
        S: RaftStorage<D, R, E>,
{
    #[tracing::instrument(skip(self, rpc, _ctx))]
    fn append_entries(
        &self,
        rpc: raft_protocol::AppendEntriesRequest<D>,
        _ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = raft_protocol::AppendEntriesResponse, Error = ()>> {
        Box::new(
            fut::wrap_future::<_, Node<D>>(self.submit_to_raft(rpc))
                .and_then(|res, _, _| fut::result(res))
        )
        // debug!(
        //     proximity = ?self, raft_command = "AppendEntries",
        //     "Submitting Raft RPC to local Raft."
        // );
        //
        // Box::new(
        //     fut::wrap_future::<_, Node<D>>(self.raft.send(msg))
        //         .map_err(|err, n, _| {
        //             error!(
        //                 local_id = n.local_id, proximity = ?n.proximity, error = ?err,
        //                 "Raft AppendEntries RPC failed in actor send."
        //             );
        //             ()
        //         })
        //         .and_then(|res, _, _| fut::result(res))
        // )
    }

    #[tracing::instrument(skip(self, rpc, _ctx))]
    fn install_snapshot(
        &self,
        rpc: raft_protocol::InstallSnapshotRequest,
        _ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = raft_protocol::InstallSnapshotResponse, Error = ()>> {
        Box::new(
            fut::wrap_future::<_, Node<D>>(self.submit_to_raft(rpc))
                .and_then(|res, _, _| fut::result(res))
        )
        // debug!(
        //     proximity = ?self, raft_command = "InstallSnapshot",
        //     "Submitting Raft RPC to local Raft."
        // );
        //
        // Box::new(
        //     fut::wrap_future::<_, Node<D>>(self.raft.send(rpc))
        //         .map_err(|err, n, _| {
        //             error!(
        //                 local_id = n.local_id, proximity = ?n.proximity, error = ?err,
        //                 "Raft AppendEntries RPC failed in actor send."
        //             );
        //             ()
        //         })
        //         .and_then(|res, _, _| fut::result(res))
        // )
    }

    #[tracing::instrument(skip(self, rpc, _ctx))]
    fn vote(
        &self,
        rpc: raft_protocol::VoteRequest,
        _ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = raft_protocol::VoteResponse, Error = ()>> {
        Box::new(
            fut::wrap_future::<_, Node<D>>(self.submit_to_raft(rpc))
                .and_then(|res, _, _| fut::result(res))
        )
        // debug!(
        //     proximity = ?self, raft_command = "VoteRequest",
        //     "Submitting Raft RPC to local Raft."
        // );
        //
        // Box::new(
        //     fut::wrap_future::<_, Node<D>>(self.raft.send(rpc))
        //         .map_err(|err, n, _| {
        //             error!(
        //                 local_id = n.local_id, proximity = ?n.proximity, error = ?err,
        //                 "Raft VoteRequest RPC failed in actor send."
        //             );
        //             ()
        //         })
        //         .and_then(|res, _, _| fut::result(res))
        // )
    }
}

impl<D, R, E, S> LocalNode<D, R, E, S>
    where
        D: AppData,
        R: AppDataResponse,
        E: AppError,
        S: RaftStorage<D, R, E>,
{
    #[tracing::instrument(skip(self))]
    fn submit_to_raft<C>(&self, rpc: C) -> Box<dyn Future<Item = C::Result, Error = ()>>
    where
        C: Message + Send + std::fmt::Debug + 'static,
        C::Result: Send,
        Raft<D, R, E, S>: Handler<C>,
        <Raft<D, R, E, S> as Actor>::Context: actix::dev::ToEnvelope<Raft<D, R, E, S>, C>
    {
        let local_id = self.id;
        let proximity = self.clone();
        debug!(?proximity, raft_rpc = ?rpc, "Submitting Raft RPC to local Raft.");

        Box::new(
            self.raft.send(rpc)
                .map_err(move |err| {
                    error!(local_id, ?proximity, error = ?err,"Raft RPC failed in actor send.");
                    ()
                })
        )
    }
}

impl<D: AppData> RaftProtocolBehavior<D> for RemoteNode {
    #[tracing::instrument(skip(self, rpc, ctx))]
    fn append_entries(
        &self,
        rpc: raft_protocol::AppendEntriesRequest<D>,
        ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = raft_protocol::AppendEntriesResponse, Error = ()>> {
        self.send_raft_command::<
            D,
            raft_protocol::AppendEntriesRequest<D>,
            raft_protocol::AppendEntriesResponse,
            port_entities::RaftAppendEntriesRequest,
            port_entities::RaftAppendEntriesResponse
        >(rpc, "entries", ctx)
    }

    #[tracing::instrument(skip(self, rpc, ctx))]
    fn install_snapshot(
        &self,
        rpc: raft_protocol::InstallSnapshotRequest,
        ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = raft_protocol::InstallSnapshotResponse, Error = ()>> {
        self.send_raft_command::<
            D,
            raft_protocol::InstallSnapshotRequest,
            raft_protocol::InstallSnapshotResponse,
            port_entities::RaftInstallSnapshotRequest,
            port_entities::RaftInstallSnapshotResponse
        >(rpc, "snapshots", ctx)
    }

    #[tracing::instrument(skip(self, rpc, ctx))]
    fn vote(
        &self,
        rpc: raft_protocol::VoteRequest,
        ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = raft_protocol::VoteResponse, Error = ()>> {
        self.send_raft_command::<
            D,
            raft_protocol::VoteRequest,
            raft_protocol::VoteResponse,
            port_entities::RaftVoteRequest,
            port_entities::RaftVoteResponse
        >(rpc, "vote", ctx)
    }
}

impl RemoteNode {
    #[tracing::instrument(skip(self, _ctx))]
    fn send_raft_command<D, M, R, M0, R0>(
        &self,
        command: M,
        path: &str,
        _ctx: &mut <Node<D> as Actor>::Context
    ) -> Box<dyn ActorFuture<Actor = Node<D>, Item = R, Error = ()>>
    where
        D: AppData,
        M: Message + std::fmt::Debug,
        M: Into<M0>,
        R: 'static,
        M0: Serialize,
        R0: for<'de> Deserialize<'de>,
        R0: Into<R>,
    {
        let route = format!("{}/{}", self.scope(), path);
        debug!(proximity = ?self, ?command, %route, "send Raft RPC to remote node.");

        let body: M0 = command.into();

        let task = self.client
            .post(&route)
            .json(&body)
            .send()
            .and_then(|mut resp| resp.json::<R0>())
            .map_err(|err| {
                //todo handle redirect, which indicates leader, but this should be reflected in raft_metrics and adjusted via consensus
                self.log_raft_protocol_error(err);
                ()
            })
            .and_then(|resp| Ok(resp.into()));

        Box::new(fut::result(task))
    }

    fn log_raft_protocol_error(&self, error: reqwest::Error) {
        match error {
            e if e.is_timeout() => {
                warn!(proximity = ?self, error = ?e, "Raft RPC to remote timed out.");
            },

            e if e.is_serialization() => {
                error!(proximity = ?self, error = ?e, "Raft RPC serialization failure.");
                panic!(e); //todo: panic to identify intermittent test error log that I think occurs on failing to parse json response.
            },

            // e if e.is_timeout() => NodeError::Timeout(e.to_string()),
            // e if e.is_client_error() => NodeError::RequestError(e),
            // e if e.is_serialization() => NodeError::ResponseFailure(e.to_string()),
            // e if e.is_http() => NodeError::RequestError(e),
            // e if e.is_server_error() => {},
            // e if e.is_redirect() => {

            e => {
                error!(proximity = ?self, error = ?e, "Raft RPC to remote failed");
            },
        }
    }
}
