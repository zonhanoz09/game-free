use super::*;

pub fn spawn_floating_text(
    commands: &mut Commands,
    rng: &mut BattleRng,
    pos: Vec2,
    text: &str,
    color: Color,
    font_size: f32,
) {
    let offset_x = rng.random_range(-14.0, 14.0);
    let offset_y = rng.random_range(16.0, 28.0);
    let vx = rng.random_range(-22.0, 22.0);
    let vy = rng.random_range(52.0, 78.0);

    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(pos.x + offset_x, pos.y + offset_y, 80.0),
        FloatingText2d {
            velocity: Vec2::new(vx, vy),
            timer: Timer::from_seconds(0.75, TimerMode::Once),
        },
    ));
}

pub fn update_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut FloatingText2d)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    for (entity, mut transform, mut text_color, mut float) in query.iter_mut() {
        float.timer.tick(std::time::Duration::from_secs_f32(dt));

        transform.translation.x += float.velocity.x * dt;
        transform.translation.y += float.velocity.y * dt;
        float.velocity.y -= 75.0 * dt;

        let progress = float.timer.fraction();

        let scale = if progress < 0.20 {
            0.6 + (progress / 0.20) * 0.5
        } else {
            1.1 - (progress - 0.20) * 0.25
        };
        transform.scale = Vec3::splat(scale);

        if progress > 0.60 {
            let alpha = 1.0 - (progress - 0.60) / 0.40;
            let current = text_color.0.to_srgba();
            text_color.0 = Color::srgba(
                current.red,
                current.green,
                current.blue,
                alpha.clamp(0.0, 1.0),
            );
        }

        if float.timer.finished() {
            if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
        }
    }
}

pub fn spawn_impact_sparks(
    commands: &mut Commands,
    rng: &mut BattleRng,
    pos: Vec2,
    color: Color,
    count: usize,
) {
    for _ in 0..count {
        let angle = rng.random_range(0.0, PI * 2.0);
        let speed = rng.random_range(40.0, 120.0);
        let vx = angle.cos() * speed;
        let vy = angle.sin() * speed;

        commands.spawn((
            Sprite {
                custom_size: Some(Vec2::splat(rng.random_range(3.0, 6.0))),
                color,
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 60.0),
            SparkParticle2d {
                velocity: Vec2::new(vx, vy),
                timer: Timer::from_seconds(rng.random_range(0.20, 0.38), TimerMode::Once),
                initial_color: color,
            },
        ));
    }
}

pub fn update_spark_particles(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut SparkParticle2d)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    for (entity, mut transform, mut sprite, mut spark) in query.iter_mut() {
        spark.timer.tick(std::time::Duration::from_secs_f32(dt));

        transform.translation.x += spark.velocity.x * dt;
        transform.translation.y += spark.velocity.y * dt;
        spark.velocity.y -= 140.0 * dt;
        spark.velocity.x *= 0.94;

        let progress = spark.timer.fraction();
        let alpha = (1.0 - progress).clamp(0.0, 1.0);
        let c = spark.initial_color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);

        if spark.timer.finished() {
            if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
        }
    }
}

pub fn update_combat_vfx(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut CombatVfx2d)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    for (entity, mut transform, mut sprite, mut vfx) in query.iter_mut() {
        vfx.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = vfx.timer.fraction();

        let cur_scale = vfx.initial_scale.lerp(vfx.target_scale, progress);
        transform.scale = Vec3::new(cur_scale.x, cur_scale.y, 1.0);

        transform.rotate_z(vfx.rotate_speed * dt);

        let alpha = (1.0 - progress).clamp(0.0, 1.0);
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);

        if vfx.timer.finished() {
            if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
        }
    }
}

pub fn update_hit_recoil(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut UnitHitRecoil2d)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    for (entity, mut transform, mut recoil) in query.iter_mut() {
        recoil.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = recoil.timer.fraction();

        let t = progress * PI * 2.0;
        let damping = 1.0 - progress;
        let offset = recoil.recoil_offset * (t.sin() * damping * 0.7);

        transform.translation.x = recoil.original_pos.x + offset.x;
        transform.translation.y = recoil.original_pos.y + offset.y;

        if recoil.timer.finished() {
            transform.translation.x = recoil.original_pos.x;
            transform.translation.y = recoil.original_pos.y;
            if let Some(mut e) = commands.get_entity(entity) { e.remove::<UnitHitRecoil2d>(); }
        }
    }
}

pub fn update_turn_spotlight(
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(&mut Transform, &mut Sprite), With<ActiveTurnSpotlight2d>>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    let t = time.elapsed_secs() * speed.multiplier;
    for (mut transform, mut sprite) in query.iter_mut() {
        transform.rotate_z(2.0 * dt);
        let pulse = (t * 5.0).sin() * 0.12 + 1.0;
        transform.scale = Vec3::splat(pulse);

        let alpha = (t * 4.0).sin() * 0.2 + 0.65;
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}
