// Peer message structs
pub mod file_search_response;
pub mod folder_contents;
pub mod place_in_queue;
pub mod place_in_queue_request;
pub mod queue_upload;
pub mod shared_file_list;
pub mod transfer_request;
pub mod transfer_response;
pub mod upload_denied;
pub mod upload_failed;
pub mod user_info;

use crate::codec::SlskRead;
use crate::error::ProtoError;
use bytes::{Buf, Bytes};

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
    TransferResponse(transfer_response::TransferResponse),
    DownloadResponse, // DEPRECATED
    UploadResponse,   // TODO
    QueueUpload(queue_upload::QueueUpload),
    PlaceInQueueRequest(place_in_queue_request::PlaceInQueueRequest),
    PlaceInQueueResponse(place_in_queue::PlaceInQueueResponse),
    UploadFailed(upload_failed::UploadFailed),
    UploadDenied(upload_denied::UploadDenied),
    FileSearchResponse(file_search_response::FileSearchResponse),
    Unknown(u32, Bytes),
}

impl PeerMessage {
    pub fn decode(code: u32, payload: &mut impl Buf) -> Result<Self, ProtoError> {
        match code {
            shared_file_list::CODE => Ok(Self::SharedFileListRequest(
                shared_file_list::SharedFileListRequest::read(payload)?,
            )),
            user_info::CODE => Ok(Self::UserInfoRequest(user_info::UserInfoRequest::read(
                payload,
            )?)),
            folder_contents::CODE => Ok(Self::FolderContentsRequest(
                folder_contents::FolderContentsRequest::read(payload)?,
            )),
            transfer_request::CODE => Ok(Self::TransferRequest(
                transfer_request::TransferRequest::read(payload)?,
            )),
            transfer_response::CODE => Ok(Self::TransferResponse(
                transfer_response::TransferResponse::read(payload)?,
            )),
            queue_upload::CODE => Ok(Self::QueueUpload(queue_upload::QueueUpload::read(payload)?)),
            place_in_queue_request::CODE => Ok(Self::PlaceInQueueRequest(
                place_in_queue_request::PlaceInQueueRequest::read(payload)?,
            )),
            place_in_queue::CODE => Ok(Self::PlaceInQueueResponse(
                place_in_queue::PlaceInQueueResponse::read(payload)?,
            )),
            upload_failed::CODE => Ok(Self::UploadFailed(upload_failed::UploadFailed::read(
                payload,
            )?)),
            upload_denied::CODE => Ok(Self::UploadDenied(upload_denied::UploadDenied::read(
                payload,
            )?)),
            file_search_response::CODE => Ok(Self::FileSearchResponse(
                file_search_response::FileSearchResponse::read(payload)?,
            )),
            other => Ok(Self::Unknown(
                other,
                payload.copy_to_bytes(payload.remaining()),
            )),
        }
    }
}
