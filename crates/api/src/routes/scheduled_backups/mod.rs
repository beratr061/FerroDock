mod backups;
mod destinations;
mod volumes;

use axum::Router;
use axum::routing::{delete, get, post};
use utoipa::OpenApi;

use self::backups::{
    __path_create_database_backup, __path_delete_backup_policy, __path_delete_database_backup,
    __path_get_backup_policy, __path_list_scheduled_backups, __path_restore_database_backup,
    __path_set_backup_policy,
};
use self::destinations::{
    __path_create_backup_destination, __path_delete_backup_destination,
    __path_list_backup_destinations, __path_test_backup_destination,
};
use self::volumes::{
    __path_create_volume_backup, __path_delete_volume_backup_policy,
    __path_get_volume_backup_policy, __path_list_volume_backups, __path_set_volume_backup_policy,
};
use crate::dto::{
    BackupDestinationDto, BackupPolicyDto, BackupPolicyRequest, BackupRunDto,
    CreateBackupDestinationRequest, CreateBackupRequest, DestinationKindDto, S3DestinationRequest,
};
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_backup_destinations,
        create_backup_destination,
        test_backup_destination,
        delete_backup_destination,
        get_backup_policy,
        set_backup_policy,
        delete_backup_policy,
        list_scheduled_backups,
        create_database_backup,
        restore_database_backup,
        delete_database_backup,
        get_volume_backup_policy,
        set_volume_backup_policy,
        delete_volume_backup_policy,
        list_volume_backups,
        create_volume_backup
    ),
    components(schemas(
        BackupDestinationDto,
        BackupPolicyDto,
        BackupPolicyRequest,
        BackupRunDto,
        CreateBackupDestinationRequest,
        CreateBackupRequest,
        DestinationKindDto,
        S3DestinationRequest
    ))
)]
pub(crate) struct ScheduledBackupApi;

pub(crate) fn scheduled_backup_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/backup-destinations",
            get(destinations::list_backup_destinations)
                .post(destinations::create_backup_destination),
        )
        .route(
            "/backup-destinations/{id}",
            delete(destinations::delete_backup_destination),
        )
        .route(
            "/backup-destinations/{id}/test",
            post(destinations::test_backup_destination),
        )
        .route(
            "/databases/{id}/backup-policy",
            get(backups::get_backup_policy)
                .put(backups::set_backup_policy)
                .delete(backups::delete_backup_policy),
        )
        .route(
            "/databases/{id}/backups",
            get(backups::list_scheduled_backups).post(backups::create_database_backup),
        )
        .route("/backup-runs/{id}", delete(backups::delete_database_backup))
        .route(
            "/backup-runs/{id}/restore",
            post(backups::restore_database_backup),
        )
        .route(
            "/applications/{id}/volumes/{volume}/backup-policy",
            get(volumes::get_volume_backup_policy)
                .put(volumes::set_volume_backup_policy)
                .delete(volumes::delete_volume_backup_policy),
        )
        .route(
            "/applications/{id}/volumes/{volume}/backups",
            get(volumes::list_volume_backups).post(volumes::create_volume_backup),
        )
}
