// Server message structs
pub mod login;
pub mod set_wait_port;
pub mod get_peer_address;
pub mod watch_user;
pub mod unwatch_user;
pub mod get_user_status;
pub mod connect_to_peer;
pub mod server_ping;
pub mod relogged;
pub mod cant_connect_to_peer;
pub mod cant_create_room;
pub mod file_search;
pub mod user_search;
pub mod wishlist_search;
pub mod wishlist_interval;
pub mod shared_folders_files;
pub mod get_user_stats;
pub mod send_upload_speed;
pub mod say_chatroom;
pub mod join_room;
pub mod leave_room;
pub mod message_user;
pub mod message_acked;
pub mod set_status;
pub mod user_joined_room;
pub mod user_left_room;
pub mod room_search;
pub mod room_list;
pub mod privileged_users;
pub mod excluded_search_phrases;
pub mod add_thing_i_like;
pub mod remove_thing_i_like;
pub mod have_no_parent;
pub mod possible_parents;
pub mod accept_children;
pub mod branch_level;
pub mod branch_root;
pub mod reset_distributed;
pub mod embedded_message;
pub mod global_room_message;
pub mod check_privileges;
pub mod room_members;
pub mod add_room_member;
pub mod remove_room_member;
pub mod recommendations;
pub mod global_recommendations;
pub mod user_interests;
pub mod similar_users;
pub mod admin_message;
pub mod give_privileges;
pub mod parent_min_speed;
pub mod parent_speed_ratio;

use bytes::{Buf, Bytes};
use crate::codec::SlskRead;
use crate::error::ProtoError;

/// Top-level server message dispatcher
#[derive(Debug)]
pub enum ServerMessage {
    Login(login::LoginResponse),
    SetWaitPort, // send-only
    GetPeerAddress(get_peer_address::GetPeerAddressResponse),
    WatchUser(watch_user::WatchUserResponse),
    UnwatchUser, // send-only
    GetUserStatus(get_user_status::GetUserStatusResponse),
    SayChatroom(say_chatroom::SayChatroomResponse),
    JoinRoom(join_room::JoinRoomResponse),
    LeaveRoom, // send-only
    UserJoinedRoom(user_joined_room::UserJoinedRoom),
    UserLeftRoom(user_left_room::UserLeftRoom),
    ConnectToPeer(connect_to_peer::ConnectToPeerResponse),
    MessageUser(message_user::MessageUserResponse),
    MessageAcked, // send-only
    FileSearch(file_search::FileSearchResponse),
    SetStatus, // send-only
    ServerPing, // send-only
    SharedFoldersFiles, // send-only
    GetUserStats(get_user_stats::GetUserStatsResponse),
    Relogged(relogged::Relogged),
    UserSearch, // send-only
    AddThingILike, // send-only
    RemoveThingILike, // send-only
    Recommendations(recommendations::RecommendationsResponse),
    GlobalRecommendations(global_recommendations::GlobalRecommendationsResponse),
    UserInterests(user_interests::UserInterestsResponse),
    RoomList(room_list::RoomListResponse),
    AdminMessage(admin_message::AdminMessageResponse),
    PrivilegedUsers(privileged_users::PrivilegedUsersResponse),
    HaveNoParent, // send-only
    ParentMinSpeed(parent_min_speed::ParentMinSpeedResponse),
    ParentSpeedRatio(parent_speed_ratio::ParentSpeedRatioResponse),
    CheckPrivileges(check_privileges::CheckPrivilegesResponse),
    EmbeddedMessage(embedded_message::EmbeddedMessageResponse),
    AcceptChildren, // send-only
    PossibleParents(possible_parents::PossibleParentsResponse),
    WishlistSearch, // send-only
    WishlistInterval(wishlist_interval::WishlistInterval),
    SimilarUsers(similar_users::SimilarUsersResponse),
    RoomMembers(room_members::RoomMembersResponse),
    AddRoomMember(add_room_member::AddRoomMemberResponse),
    RemoveRoomMember(remove_room_member::RemoveRoomMemberResponse),
    RoomSearch, // send-only
    SendUploadSpeed, // send-only
    BranchLevel, // send-only
    BranchRoot, // send-only
    ResetDistributed(reset_distributed::ResetDistributed),
    GlobalRoomMessage(global_room_message::GlobalRoomMessageResponse),
    ExcludedSearchPhrases(excluded_search_phrases::ExcludedSearchPhrasesResponse),
    CantConnectToPeer(cant_connect_to_peer::CantConnectToPeerResponse),
    CantCreateRoom(cant_create_room::CantCreateRoom),
    GivePrivileges, // send-only
    Unknown(u32, Bytes),
}

impl ServerMessage {
    pub fn decode(code: u32, payload: &mut impl Buf) -> Result<Self, ProtoError> {
        match code {
            login::CODE => Ok(Self::Login(login::LoginResponse::read(payload)?)),
            set_wait_port::CODE => Ok(Self::SetWaitPort),
            get_peer_address::CODE => Ok(Self::GetPeerAddress(get_peer_address::GetPeerAddressResponse::read(payload)?)),
            watch_user::CODE => Ok(Self::WatchUser(watch_user::WatchUserResponse::read(payload)?)),
            unwatch_user::CODE => Ok(Self::UnwatchUser),
            get_user_status::CODE => Ok(Self::GetUserStatus(get_user_status::GetUserStatusResponse::read(payload)?)),
            say_chatroom::CODE => Ok(Self::SayChatroom(say_chatroom::SayChatroomResponse::read(payload)?)),
            join_room::CODE => Ok(Self::JoinRoom(join_room::JoinRoomResponse::read(payload)?)),
            leave_room::CODE => Ok(Self::LeaveRoom),
            user_joined_room::CODE => Ok(Self::UserJoinedRoom(user_joined_room::UserJoinedRoom::read(payload)?)),
            user_left_room::CODE => Ok(Self::UserLeftRoom(user_left_room::UserLeftRoom::read(payload)?)),
            connect_to_peer::CODE => Ok(Self::ConnectToPeer(connect_to_peer::ConnectToPeerResponse::read(payload)?)),
            message_user::CODE => Ok(Self::MessageUser(message_user::MessageUserResponse::read(payload)?)),
            message_acked::CODE => Ok(Self::MessageAcked),
            file_search::CODE => Ok(Self::FileSearch(file_search::FileSearchResponse::read(payload)?)),
            set_status::CODE => Ok(Self::SetStatus),
            server_ping::CODE => Ok(Self::ServerPing),
            shared_folders_files::CODE => Ok(Self::SharedFoldersFiles),
            get_user_stats::CODE => Ok(Self::GetUserStats(get_user_stats::GetUserStatsResponse::read(payload)?)),
            relogged::CODE => Ok(Self::Relogged(relogged::Relogged::read(payload)?)),
            user_search::CODE => Ok(Self::UserSearch),
            add_thing_i_like::CODE => Ok(Self::AddThingILike),
            remove_thing_i_like::CODE => Ok(Self::RemoveThingILike),
            recommendations::CODE => Ok(Self::Recommendations(recommendations::RecommendationsResponse::read(payload)?)),
            global_recommendations::CODE => Ok(Self::GlobalRecommendations(global_recommendations::GlobalRecommendationsResponse::read(payload)?)),
            user_interests::CODE => Ok(Self::UserInterests(user_interests::UserInterestsResponse::read(payload)?)),
            room_list::CODE => Ok(Self::RoomList(room_list::RoomListResponse::read(payload)?)),
            admin_message::CODE => Ok(Self::AdminMessage(admin_message::AdminMessageResponse::read(payload)?)),
            privileged_users::CODE => Ok(Self::PrivilegedUsers(privileged_users::PrivilegedUsersResponse::read(payload)?)),
            have_no_parent::CODE => Ok(Self::HaveNoParent),
            parent_min_speed::CODE => Ok(Self::ParentMinSpeed(parent_min_speed::ParentMinSpeedResponse::read(payload)?)),
            parent_speed_ratio::CODE => Ok(Self::ParentSpeedRatio(parent_speed_ratio::ParentSpeedRatioResponse::read(payload)?)),
            check_privileges::CODE => Ok(Self::CheckPrivileges(check_privileges::CheckPrivilegesResponse::read(payload)?)),
            embedded_message::CODE => Ok(Self::EmbeddedMessage(embedded_message::EmbeddedMessageResponse::read(payload)?)),
            accept_children::CODE => Ok(Self::AcceptChildren),
            possible_parents::CODE => Ok(Self::PossibleParents(possible_parents::PossibleParentsResponse::read(payload)?)),
            wishlist_search::CODE => Ok(Self::WishlistSearch),
            wishlist_interval::CODE => Ok(Self::WishlistInterval(wishlist_interval::WishlistInterval::read(payload)?)),
            similar_users::CODE => Ok(Self::SimilarUsers(similar_users::SimilarUsersResponse::read(payload)?)),
            room_members::CODE => Ok(Self::RoomMembers(room_members::RoomMembersResponse::read(payload)?)),
            add_room_member::CODE => Ok(Self::AddRoomMember(add_room_member::AddRoomMemberResponse::read(payload)?)),
            remove_room_member::CODE => Ok(Self::RemoveRoomMember(remove_room_member::RemoveRoomMemberResponse::read(payload)?)),
            room_search::CODE => Ok(Self::RoomSearch),
            send_upload_speed::CODE => Ok(Self::SendUploadSpeed),
            branch_level::CODE => Ok(Self::BranchLevel),
            branch_root::CODE => Ok(Self::BranchRoot),
            reset_distributed::CODE => Ok(Self::ResetDistributed(reset_distributed::ResetDistributed::read(payload)?)),
            global_room_message::CODE => Ok(Self::GlobalRoomMessage(global_room_message::GlobalRoomMessageResponse::read(payload)?)),
            excluded_search_phrases::CODE => Ok(Self::ExcludedSearchPhrases(excluded_search_phrases::ExcludedSearchPhrasesResponse::read(payload)?)),
            cant_connect_to_peer::CODE => Ok(Self::CantConnectToPeer(cant_connect_to_peer::CantConnectToPeerResponse::read(payload)?)),
            cant_create_room::CODE => Ok(Self::CantCreateRoom(cant_create_room::CantCreateRoom::read(payload)?)),
            give_privileges::CODE => Ok(Self::GivePrivileges),
            other => Ok(Self::Unknown(other, payload.copy_to_bytes(payload.remaining()))),
        }
    }
}
