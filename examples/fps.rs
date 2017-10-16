// This is a more-or-less complete example, implementing a protocol for
// hypothetic first person shooter.

// Basic types
#[qlinspace(low="-1.2e40",high="1e4",step="0.1")]
type XZ = f32;

#[qlinspace(low="-100", high="100", step="0.1")]
type Y = f32;

#[qlinspace(low="-30", high="30", step="0.1")]
type OffsetCoord = f32;

#[qwrap(low="0", high="6.28318", step="0.01")]
type Yaw = f32;

#[qwrap(low="-3.14159", high="3.14159", step="0.05")]
type Pitch = f32;

#[qlinspace(low="0", high="1e4", step="0.001")]
type Time = f32;

struct WorldPosition {
    x: XZ,
    y: Y,
    z: XZ
}

struct Offset {
    x: OffsetCoord,
    y: OffsetCoord,
    z: OffsetCoord
}

struct YawPitch {
    yaw: Yaw,
    pitch: Pitch
}

// Movement state
#[qlinspace("-10", "100", step="0.1")]
type Stamina = f32;

enum MovementMode {
    Run,
    Crouch,
    Sprint,
    Knockdown
}

struct MovementState {
    position: WorldPosition,
    yaw_pitch: YawPitch,
    velocity: Offset,

    fall_velocity: OffsetCoord,
    movement_mode: MovementMode,

    fire_position: WorldPosition,

    current_stamina: Stamina,
    was_sprinting: bool
}

// Camera state
enum DeviationType {
    Sway,
    Bullet,
    Grenade
}

enum CameraMode {
    Sniper,
    Zoom,
    Standard
}

#[qlinspace("-1", "1", step="0.001")]
type Amplitude = f32;

#[qlinspace("0", "160", step="0.5")]
type Fov = f32;

struct DeviatorState {
    amplitude: Amplitude,
    time: Time
}

struct CameraState {
    mode: CameraMode,

    yaw_pitch: YawPitch,
    deviated_yaw_pitch: YawPitch,

    distance: OffsetCoord,
    fov: Fov,
    additional_distance: OffsetCoord,

    kick_up: Amplitude,

    target_point: WorldPosition,

    deviator_sway: DeviatorState,
    deviator_bullet: DeviatorState,
    deviator_grenade: DeviatorState
}

// Projectiles state
struct BulletState {
    active: bool,
    affected_by_gravity: bool,

    timestamp: Time,
    origin: WorldPosition,
    velocity: Offset
}

struct ProjectilesState {
    bullets: [BulletState; 64]
}

// Health state
#[qlinspace("0", "100", step="0.5")]
type Health = f32;

#[ibound("0", "10")]
type HumanID = u32;

#[ibound("0", "5")]
type ReviveMedKitCount = u32;

#[qlinspace("-1", "100", step="0.1")]
type RemainingTime = f32;

enum LiveState {
    Alive,
    KnockedDown,
    Dead,
    CompletelyDead
}

struct HealthState {
    health: Health,

    current_max_health: Health,
    max_health: Health,

    revive_progress: Health,

    reviver_player_id: HumanID,
    revived_player_id: HumanID,

    remaining_revives: ReviveMedKitCount,
    remaining_medkits: ReviveMedKitCount,

    live_state: LiveState,

    knockdown_remain_time: RemainingTime,

    heal_remain_amount: Health,
    heal_remain_cooldown_time: RemainingTime,

    last_damage_direction: Offset,
    last_damage_timestamp: Time,
}

// Abilities state
struct AbilitiesState {
    target_human_id: HumanID,

    remain_active_time: RemainingTime,
    remain_cooldown_time: RemainingTime,

    suppress_buff_remaining_time: RemainingTime,
    scan_buff_remaining_time: RemainingTime,
    concentration_buff_remaining_time: RemainingTime
}

// Human state
struct HumanState {
    movement: MovementState,
    camera: CameraState,
    projectiles: ProjectilesState,
    abilities: AbilitiesState
}
