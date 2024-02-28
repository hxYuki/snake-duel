use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rand::prelude::*;
use components::{DieTickTimer, PlayerInput, SnakeEntity, SnakeMovement};
use events::{SnakeDiedEvent, SnakeGrowthEvent, SpawnSnakeEvent};
use rand::prelude::*;
use resources::{Cell, Grid};

mod bundles;
mod components;
mod events;
mod resources;
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EntropyPlugin::<WyRand>::default()))
        .add_systems(Startup, (setup,).chain())
        .add_systems(
            Update,
            place_food.run_if(on_timer(Duration::from_secs_f32(5.))),
        )
        .add_systems(Update, place_snake)
        .add_event::<SnakeGrowthEvent>()
        .add_event::<SnakeDiedEvent>()
        .add_event::<SpawnSnakeEvent>()
        .add_systems(
            Update,
            (
                snake_input,
                snake_movement_timer_tick,
                snake_eat,
                snake_blocked,
                snake_die,
                snake_grow,
                snake_move,
                draw,
            )
                .chain(),
        )
        .run();
}
fn setup(mut commands: Commands, mut spawn_snake: EventWriter<SpawnSnakeEvent>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(12. * 16., 12. * 16., 1000.),
        ..Default::default()
    });
    let mut grid = Grid::new(25, 25);
    grid.build_wall();
    commands.insert_resource(grid);

    spawn_snake.send(SpawnSnakeEvent {
        position: IVec2::new(10, 10),
        level: 0,
    });
}

fn draw(grid: Res<Grid>, mut gizmos: Gizmos) {
    const GRID_SIZE: f32 = 16.;
    grid.grid.iter().enumerate().for_each(|(i, cell)| {
        let row = i / grid.width;
        let col = i % grid.width;
        let position = Vec2::new(col as f32, row as f32) * GRID_SIZE;
        match cell {
            resources::Cell::Wall => {
                gizmos.rect_2d(position, 0., Vec2::ONE * GRID_SIZE, Color::GRAY)
            }
            resources::Cell::Head(_) => {
                gizmos.rect_2d(position, 0., Vec2::ONE * GRID_SIZE, Color::RED)
            }
            resources::Cell::Body(_, _) => {
                gizmos.rect_2d(position, 0., Vec2::ONE * GRID_SIZE, Color::ORANGE_RED)
            }
            resources::Cell::Food => {
                gizmos.rect_2d(position, 0., Vec2::ONE * GRID_SIZE / 2., Color::GREEN)
            }
            _ => {}
        }
    });
}

fn place_snake(
    mut grid: ResMut<Grid>,
    mut commands: Commands,
    mut spawn_snake: EventReader<SpawnSnakeEvent>,
    player_snake: Query<Entity, (With<SnakeEntity>, With<PlayerInput>)>,
) {
    spawn_snake.read().for_each(|se| {
        let spawn_pos = se.position;
        let snake_ett = components::SnakeEntity {
            length: 6,
            direction: IVec2::new(1, 0),
            position: spawn_pos,
            tail: spawn_pos,
            level: se.level,
        };
        let snake_id = if player_snake.is_empty() {
            commands
                .spawn(bundles::PlayerControlSnakeBundle::from(snake_ett))
                .id()
        } else {
            commands
                .spawn(bundles::AutomaticControlSnakeBundle::from(snake_ett))
                .id()
        };

        grid.set(spawn_pos, resources::Cell::Head(snake_id));
    })
}

fn place_food(mut grid: ResMut<Grid>, mut rand: ResMut<GlobalEntropy<WyRand>>) {
    let Some(food_pos) = grid
        .grid
        .iter()
        .enumerate()
        .filter(|(_, cell)| matches!(cell, Cell::Empty))
        .choose(rand.as_mut())
        .map(|(i, _)| {
            let x = i % grid.width;
            let y = i / grid.width;
            (x as i32, y as i32)
        })
    else {
        return;
    };
    grid.set(food_pos.into(), resources::Cell::Food);
}

fn snake_movement_timer_tick(time: Res<Time>, mut snakes: Query<&mut components::SnakeMovement>) {
    snakes.par_iter_mut().for_each(|mut movement| {
        if movement.interval != movement.move_timer.duration() {
            let i = movement.interval;
            if movement.move_timer.elapsed() > i {
                movement.move_timer.set_elapsed(i);
            }
            movement.move_timer.set_duration(i);
        }

        movement.move_timer.tick(time.delta());
    })
}

fn snake_eat(
    grid: Res<Grid>,
    mut snakes: Query<(
        Entity,
        &components::SnakeEntity,
        &mut components::SnakeMovement,
    )>,
    mut growth_events: EventWriter<SnakeGrowthEvent>,
) {
    snakes.iter_mut().for_each(|(ett, snake, mut movement)| {
        if !movement.move_timer.just_finished() {
            return;
        }
        let next_move = snake.position + snake.direction;

        match grid.get(next_move).unwrap_or(resources::Cell::Wall) {
            resources::Cell::Empty => {movement.blocked = false;}          // Move
            resources::Cell::Food => {
                movement.blocked = false;
                growth_events.send(SnakeGrowthEvent{snake:ett});
            },       // Grow and move
            resources::Cell::Head(_) |  // dual! and stop
            resources::Cell::Body(_, _) | // stop and die
            resources::Cell::Wall => {movement.blocked = true;},       // stop and die
        }
    })
}
fn snake_blocked(
    mut snakes: Query<(Entity, &SnakeMovement, &mut DieTickTimer)>,
    time: Res<Time>,
    mut died_event: EventWriter<SnakeDiedEvent>,
) {
    snakes
        .iter_mut()
        .for_each(|(ett, movement, mut die_timer)| {
            if movement.blocked {
                die_timer.tick(time.delta());
            } else {
                let elapse = die_timer.elapsed().saturating_sub(time.delta());
                die_timer.set_elapsed(elapse);
            }
            if die_timer.just_finished() {
                died_event.send(SnakeDiedEvent { snake: ett });
            }
        });
}

fn snake_grow(
    mut snakes: Query<(&mut SnakeEntity)>,
    mut growth_event: EventReader<SnakeGrowthEvent>,
) {
    growth_event.read().for_each(|ge| {
        snakes.get_mut(ge.snake).unwrap().length += 1;
    })
}

fn snake_die(
    mut died_event: EventReader<SnakeDiedEvent>,
    snakes: Query<&SnakeEntity>,
    mut grid: ResMut<Grid>,
    mut rand: ResMut<GlobalEntropy<WyRand>>,
    mut commands: Commands,
) {
    died_event.read().for_each(|de| {
        let snake = snakes.get(de.snake).unwrap();
        let mut snake_occupied = vec![];
        let mut tail_pos = snake.tail;
        while tail_pos != snake.position {
            let next = grid.get(tail_pos).unwrap();
            snake_occupied.push(tail_pos);
            grid.set(tail_pos, resources::Cell::Empty);

            let resources::Cell::Body(_, next_pos) = next else {
                return;
            };
            tail_pos = next_pos;
        }
        snake_occupied.push(tail_pos);
        grid.set(tail_pos, resources::Cell::Empty);

        snake_occupied
            .choose_multiple(rand.as_mut(), snake.length / 3)
            .for_each(|pos| {
                grid.set(*pos, resources::Cell::Food);
            });
        commands.entity(de.snake).despawn();
    })
}

fn snake_move(
    mut grid: ResMut<Grid>,
    mut snakes: Query<(
        Entity,
        &mut components::SnakeEntity,
        &components::SnakeMovement,
    )>,
) {
    snakes
        .iter_mut()
        .for_each(|(ett, mut snake_entity, snake_movement)| {
            if !snake_movement.move_timer.just_finished() || snake_movement.blocked {
                return;
            }
            let next_move = snake_entity.position + snake_entity.direction;

            grid.set(next_move, resources::Cell::Head(ett));

            grid.set(snake_entity.position, resources::Cell::Body(ett, next_move));

            let mut tail_pos = snake_entity.position;

            for i in 0..snake_entity.length - 1 {
                let Some(next_pos) =
                    grid.find_around(tail_pos, resources::Cell::Body(ett, tail_pos))
                else {
                    break;
                };

                if i == snake_entity.length - 2 {
                    grid.set(next_pos, resources::Cell::Empty);
                } else {
                    tail_pos = next_pos;
                }
            }

            snake_entity.position = next_move;
            snake_entity.tail = tail_pos;
        })
}

fn snake_input(
    grid: Res<Grid>,
    mut snake_q: Query<
        (Entity, &mut components::SnakeEntity, &mut SnakeMovement),
        With<components::PlayerInput>,
    >,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Some((ett, mut snake, mut movement)) = snake_q.get_single_mut().ok() else {
        return;
    };
    let dir = input
        .get_just_pressed()
        .map(|key| match key {
            KeyCode::KeyW | KeyCode::ArrowUp => IVec2::new(0, 1),
            KeyCode::KeyA | KeyCode::ArrowLeft => IVec2::new(-1, 0),
            KeyCode::KeyS | KeyCode::ArrowDown => IVec2::new(0, -1),
            KeyCode::KeyD | KeyCode::ArrowRight => IVec2::new(1, 0),
            _ => IVec2::ZERO,
        })
        .last()
        .unwrap_or(IVec2::ZERO);

    if dir != IVec2::ZERO
        && grid.get(snake.position + dir) != Some(resources::Cell::Body(ett, snake.position))
    {
        snake.direction = dir;
    }

    let interval = if input.pressed(KeyCode::KeyJ) {
        0.1
    } else {
        0.15
    };
    movement.interval = Duration::from_secs_f64(interval);
}
