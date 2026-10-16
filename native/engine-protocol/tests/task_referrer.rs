//! `TaskDto` 必须在 wire 上保留任务来源页与加速来源累计。

use fluxdown_engine::model::{SourceBytes, TaskInfo};
use fluxdown_engine::transfer_activity::{BtPeerStats, TaskRuntime};

#[test]
fn task_dto_json_carries_referrer() -> Result<(), serde_json::Error> {
    let info = TaskInfo {
        task_id: "t1".to_owned(),
        url: "https://example.com/f.zip".to_owned(),
        file_name: "f.zip".to_owned(),
        save_dir: "/tmp".to_owned(),
        status: 2,
        downloaded_bytes: 10,
        total_bytes: 100,
        error_message: String::new(),
        created_at: "1700000000".to_owned(),
        proxy_url: String::new(),
        queue_id: String::new(),
        checksum: String::new(),
        ignore_tls_errors: false,
        file_missing: false,
        completed_at: String::new(),
        segments: 0,
        queue_order: 0,
        uploaded_bytes: 0,
        uploaded_at_completion: 0,
        seeding_status: 0,
        seeding_message: String::new(),
        seeding_time_secs: 0,
        seed_ratio_limit_milli: -2,
        seed_post_ratio_limit_milli: -2,
        seed_time_limit_minutes: -2,
        seed_inactive_time_limit_minutes: -2,
        seed_upload_limit_bps: 2048,
        referrer: "https://example.com/page".to_owned(),
        group_id: String::new(),
        rss_source_id: String::new(),
        origin_url: String::new(),
        auto_route: String::new(),
        source_bytes: SourceBytes::default(),
    };
    let dto = fluxdown_engine_protocol::task_info_to_dto(info);
    assert_eq!(dto.referrer, "https://example.com/page");
    assert_eq!(dto.seed_upload_limit_bps, 2048);
    let json = serde_json::to_string(&dto)?;
    assert!(json.contains(r#""referrer":"https://example.com/page""#));
    assert!(json.contains(r#""seedUploadLimitBps":2048"#));
    Ok(())
}

#[test]
fn task_dto_json_carries_persisted_source_bytes() -> Result<(), serde_json::Error> {
    let info = TaskInfo {
        task_id: "t2".to_owned(),
        url: String::new(),
        file_name: String::new(),
        save_dir: String::new(),
        status: 1,
        downloaded_bytes: 0,
        total_bytes: 0,
        error_message: String::new(),
        created_at: String::new(),
        proxy_url: String::new(),
        queue_id: String::new(),
        checksum: String::new(),
        ignore_tls_errors: false,
        file_missing: false,
        completed_at: String::new(),
        segments: 0,
        queue_order: 0,
        uploaded_bytes: 0,
        uploaded_at_completion: 0,
        seeding_status: 0,
        seeding_message: String::new(),
        seeding_time_secs: 0,
        seed_ratio_limit_milli: -2,
        seed_post_ratio_limit_milli: -2,
        seed_time_limit_minutes: -2,
        seed_inactive_time_limit_minutes: -2,
        seed_upload_limit_bps: 0,
        referrer: String::new(),
        group_id: String::new(),
        rss_source_id: String::new(),
        origin_url: String::new(),
        auto_route: String::new(),
        source_bytes: SourceBytes {
            cdn: 1_000,
            proxy: 200,
            nic: 30,
        },
    };
    let dto = fluxdown_engine_protocol::task_info_to_dto(info);
    assert_eq!(
        (
            dto.source_bytes.cdn_bytes,
            dto.source_bytes.proxy_bytes,
            dto.source_bytes.nic_bytes
        ),
        (1_000, 200, 30)
    );
    let json = serde_json::to_string(&dto)?;
    assert!(json.contains(r#""sourceBytes":{"cdnBytes":1000,"proxyBytes":200,"nicBytes":30}"#));
    Ok(())
}

#[test]
fn runtime_dto_distinguishes_no_attribution_from_zero_attribution() -> Result<(), serde_json::Error>
{
    let unattributed = fluxdown_engine_protocol::task_runtime_to_dto(TaskRuntime::default());
    assert_eq!(unattributed.source_bytes, None);
    assert!(serde_json::to_string(&unattributed)?.contains(r#""sourceBytes":null"#));

    let attributed = fluxdown_engine_protocol::task_runtime_to_dto(TaskRuntime {
        source_bytes: Some(SourceBytes {
            cdn: 5,
            proxy: 0,
            nic: 9,
        }),
        ..TaskRuntime::default()
    });
    assert_eq!(
        attributed.source_bytes,
        Some(fluxdown_protocol::TaskSourceBytesDto {
            cdn_bytes: 5,
            proxy_bytes: 0,
            nic_bytes: 9,
        })
    );
    Ok(())
}

#[test]
fn runtime_dto_carries_bt_peer_lifecycle_and_quality_stats() -> Result<(), serde_json::Error> {
    let dto = fluxdown_engine_protocol::task_runtime_to_dto(TaskRuntime {
        connected_peers: Some(7),
        bt_peer_stats: Some(BtPeerStats {
            seen: 31,
            queued: 4,
            connecting: 6,
            live: 7,
            dead: 14,
            useful_peers: Some(3),
            live_tcp: Some(4),
            live_utp: Some(2),
            live_socks: Some(1),
            connection_attempts: Some(11),
            connection_successes: Some(8),
            connection_errors: Some(3),
            total_connecting_ms: Some(9_000),
            fetched_bytes: Some(10_000),
            downloaded_pieces: Some(12),
            total_piece_download_ms: Some(13_000),
            peer_steals: Some(2),
            uploaded_bytes: Some(21_000),
            uploading_peers: Some(5),
            interested_peers: Some(9),
            upload_requests: Some(44),
            upload_queue_depth: Some(6),
            incoming_rejected: Some(17),
            admission_rejected: Some(18),
        }),
        ..TaskRuntime::default()
    });

    assert_eq!(dto.connected_peers, Some(7));
    assert_eq!(
        dto.bt_peer_stats,
        Some(fluxdown_protocol::BtPeerStatsDto {
            seen: 31,
            queued: 4,
            connecting: 6,
            live: 7,
            dead: 14,
            useful_peers: Some(3),
            live_tcp: Some(4),
            live_utp: Some(2),
            live_socks: Some(1),
            connection_attempts: Some(11),
            connection_successes: Some(8),
            connection_errors: Some(3),
            total_connecting_ms: Some(9_000),
            fetched_bytes: Some(10_000),
            downloaded_pieces: Some(12),
            total_piece_download_ms: Some(13_000),
            peer_steals: Some(2),
            uploaded_bytes: Some(21_000),
            uploading_peers: Some(5),
            interested_peers: Some(9),
            upload_requests: Some(44),
            upload_queue_depth: Some(6),
            incoming_rejected: Some(17),
            admission_rejected: Some(18),
        })
    );
    assert_eq!(
        serde_json::to_string(&dto)?,
        r#"{"taskId":"","sampledAtMs":0,"sampleSequence":0,"activeTransfers":null,"connectedPeers":7,"btPeerStats":{"seen":31,"queued":4,"connecting":6,"live":7,"dead":14,"usefulPeers":3,"liveTcp":4,"liveUtp":2,"liveSocks":1,"connectionAttempts":11,"connectionSuccesses":8,"connectionErrors":3,"totalConnectingMs":9000,"fetchedBytes":10000,"downloadedPieces":12,"totalPieceDownloadMs":13000,"peerSteals":2,"uploadedBytes":21000,"uploadingPeers":5,"interestedPeers":9,"uploadRequests":44,"uploadQueueDepth":6,"incomingRejected":17,"admissionRejected":18},"parallelismLimit":null,"totalBytes":0,"segments":[],"sourceBytes":null}"#
    );
    Ok(())
}

#[test]
fn runtime_dto_accepts_old_wire_without_bt_peer_stats() -> Result<(), serde_json::Error> {
    let dto: fluxdown_protocol::TaskRuntimeDto = serde_json::from_str(
        r#"{"taskId":"t","sampledAtMs":1,"sampleSequence":2,"activeTransfers":null,"connectedPeers":null,"parallelismLimit":null,"totalBytes":0,"segments":[],"sourceBytes":null}"#,
    )?;
    assert_eq!(dto.bt_peer_stats, None);
    Ok(())
}

#[test]
fn runtime_dto_accepts_old_bt_peer_stats_without_quality_fields() -> Result<(), serde_json::Error> {
    let dto: fluxdown_protocol::TaskRuntimeDto = serde_json::from_str(
        r#"{"taskId":"t","sampledAtMs":1,"sampleSequence":2,"activeTransfers":null,"connectedPeers":2,"btPeerStats":{"seen":4,"queued":1,"connecting":2,"live":2,"dead":0},"parallelismLimit":null,"totalBytes":0,"segments":[],"sourceBytes":null}"#,
    )?;
    let stats = dto.bt_peer_stats.expect("missing BT peer stats");
    assert_eq!(stats.seen, 4);
    assert_eq!(stats.useful_peers, None);
    assert_eq!(stats.connection_attempts, None);
    assert_eq!(stats.fetched_bytes, None);
    Ok(())
}
