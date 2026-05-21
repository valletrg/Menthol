// Peer message structs
pub mod shared_file_list;
pub mod user_info;
pub mod folder_contents;
pub mod transfer_request;
pub mod queue_upload;
pub mod place_in_queue;
pub mod upload_failed;
pub mod upload_denied;
pub mod file_search_response;

use bytes::{Buf, Bytes};
use crate::codec::SlskRead;
use crate::error::ProtoError;

/// Top-level peer message dispatcher
#[derive(Debug)]
pub enum PeerMessage {
    SharedFileListRequest(shared_file_list::SharedFileListRequest),
    SharedFileListResponse, // TODO
    UserInfoRequest(user_info::UserInfoRequest),
    UserInfoResponse, // TODO
    FolderContentsRequest(folder_contents::FolderContentsRequest),
    FolderContentsResponse, // TODO
    TransferRequest(transfer_request::TransferRequest),
    DownloadResponse, // DEPRECATED
    UploadResponse, // TODO
    QueueUpload(queue_upload::QueueUpload),
    PlaceInQueueResponse(place_in_queue::PlaceInQueueResponse),
    UploadFailed(upload_failed::UploadFailed),
    UploadDenied(upload_denied::UploadDenied),
    FileSearchResponse(file_search_response::FileSearchResponse),
    Unknown(u32, Bytes),
}

impl PeerMessage {
    pub fn decode(code: u32, payload: &mut impl Buf) -> Result<Self, ProtoError> {
        match code {
            shared_file_list::CODE => Ok(Self::SharedFileListRequest(shared_file_list::SharedFileListRequest::read(payload)?)),
            user_info::CODE => Ok(Self::UserInfoRequest(user_info::UserInfoRequest::read(payload)?)),
            folder_contents::CODE => Ok(Self::FolderContentsRequest(folder_contents::FolderContentsRequest::read(payload)?)),
            transfer_request::CODE => Ok(Self::TransferRequest(transfer_request::TransferRequest::read(payload)?)),
            queue_upload::CODE => Ok(Self::QueueUpload(queue_upload::QueueUpload::read(payload)?)),
            place_in_queue::CODE => Ok(Self::PlaceInQueueResponse(place_in_queue::PlaceInQueueResponse::read(payload)?)),
            upload_failed::CODE => Ok(Self::UploadFailed(upload_failed::UploadFailed::read(payload)?)),
            upload_denied::CODE => Ok(Self::UploadDenied(upload_denied::UploadDenied::read(payload)?)),
            file_search_response::CODE => Ok(Self::FileSearchResponse(file_search_response::FileSearchResponse::read(payload)?)),
            other => Ok(Self::Unknown(other, payload.copy_to_bytes(payload.remaining()))),
        }
    }
}
