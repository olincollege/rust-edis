
use crate::messages::{requests::{
    announce_shard_request::AnnounceShardRequest,
    get_client_shard_info_request::GetClientShardInfoRequest,
    query_version_request::QueryVersionRequest,
    read_request::ReadRequest,
    write_request::WriteRequest
}, responses::{
    announce_shard_response::AnnounceShardResponse,
    get_client_shard_info_response::GetClientShardInfoResponse,
    query_version_response::QueryVersionResponse,
    read_response::ReadResponse,
    write_response::WriteResponse
}};

trait Router {

    /// Callback for handling new requests
    fn handle_announce_shard_request(req: AnnounceShardRequest);

    fn handle_get_client_shard_info_request(req: GetClientShardInfoRequest);

    fn handle_query_version_request(req: QueryVersionRequest);

    fn handle_read_request(req: ReadRequest);

    fn handle_write_request(req: WriteRequest);

    /// Functions for queueing outbound requests
    fn queue_announce_shard_request(req: AnnounceShardRequest);

    fn queue_get_client_shard_info_request(req: GetClientShardInfoRequest);

    fn queue_query_version_request(req: QueryVersionRequest);

    fn queue_read_request(req: ReadRequest);

    fn queue_write_request(req: WriteRequest);

    /// Callbacks for handling responses to outbound requests
    fn handle_announce_shard_response(res: AnnounceShardResponse);

    fn handle_get_client_shard_info_response(res: GetClientShardInfoResponse);

    fn handle_query_version_response(res: QueryVersionResponse);

    fn handle_read_response(res: ReadResponse);

    fn handle_write_response(res: WriteResponse);
}
