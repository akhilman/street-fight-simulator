use hecs::*;
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{collections::HashMap, default::Default, fmt, ops::Deref};

// Новый тип для действия собаки

#[derive(Clone, Copy, Debug)]
enum Action {
    Idle,
    Barks,
    Snarls,
    Attack,
}

impl Action {
    fn random_aggression(rng: &mut impl Rng) -> Self {
        [Self::Barks, Self::Snarls, Self::Attack]
            .choose(rng)
            .cloned()
            .unwrap()
    }
    fn idle() -> Self {
        Action::Idle
    }
}

// Флаг для живых собак

#[derive(Debug)]
struct Alive;

// Новый тип для последней атаковавшей сабокаки

#[derive(Default, Debug, Clone, Copy)]
pub struct Attacker(Option<Entity>);

impl Attacker {
    pub fn new_none() -> Self {
        Self(None)
    }
    pub fn entity(&self) -> Option<&Entity> {
        self.0.as_ref()
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl From<Option<Entity>> for Attacker {
    fn from(maybe_ent: Option<Entity>) -> Self {
        Attacker(maybe_ent)
    }
}

// Новый тип для урона от зубов

#[derive(Debug)]
struct Damage(u32);

impl Damage {
    fn value(&self) -> u32 {
        self.0
    }
}

impl Default for Damage {
    fn default() -> Self {
        Damage(0)
    }
}

impl From<u32> for Damage {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl fmt::Display for Damage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Новый тип для врага собаки

#[derive(Default, Debug, Clone, Copy)]
struct Enemy(Option<Entity>);

impl Enemy {
    pub fn new_none() -> Self {
        Self(None)
    }
    pub fn entity(&self) -> Option<&Entity> {
        self.0.as_ref()
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl From<Option<Entity>> for Enemy {
    fn from(maybe_ent: Option<Entity>) -> Self {
        Enemy(maybe_ent)
    }
}

// Новый тип для жизни собаки

#[derive(Clone, Copy, Debug)]
struct Health(u32);

impl Health {
    fn value(&self) -> u32 {
        self.0
    }
}

impl Default for Health {
    fn default() -> Self {
        Health(10)
    }
}

impl From<u32> for Health {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl fmt::Display for Health {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Новый тип для имени собаки

#[derive(Debug)]
struct Name(String);

impl Name {
    fn new<T: ToString>(name: T) -> Self {
        Name(name.to_string())
    }
}

impl From<&'_ str> for Name {
    fn from(s: &str) -> Self {
        Name(s.into())
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Name(s)
    }
}

impl Deref for Name {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

/// Заполняем мир собаками
fn populate_world<I, T>(world: &mut World, names: I)
where
    T: ToString,
    I: IntoIterator<Item = T>,
{
    // Для каждой собаки создаётся сущность со всеми необходимыми компонентами
    let _dogs: Vec<Entity> = names
        .into_iter()
        .map(|name| -> Entity {
            world.spawn((
                Alive,
                Name::new(name),
                Health::default(),
                Action::Snarls,
                Attacker::default(),
                Enemy::default(),
                Damage::default(),
            ))
        })
        .collect();
}

/// Выбираем действие для сабокаки
fn choose_action(world: &mut World, rng: &mut impl Rng) {
    world
        .query_mut::<(&mut Action, &Enemy)>()
        .into_iter()
        .for_each(|(_, (action, enemy))| {
            *action = if enemy.is_some() {
                Action::random_aggression(rng)
            } else {
                Action::idle()
            }
        })
}

/// Выбираем врагов
fn choose_enemy(world: &mut World, rng: &mut impl Rng) {
    // Собираем всех кого можно атактовать
    let targets: Vec<Entity> = world
        .query::<()>()
        .with::<Alive>()
        .into_iter()
        .map(|(e, ())| e)
        .collect();

    // Не атакуем дохлых собак
    world
        .query_mut::<&mut Enemy>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(_, enemy)| {
            if let Some(enemy_ent) = enemy.entity() {
                if !targets.contains(enemy_ent) {
                    *enemy = Enemy::new_none();
                }
            }
        });

    // Теперь выставляем целью ту псину, которая нас покусала
    world
        .query_mut::<(&mut Enemy, &Attacker)>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(_, (enemy, attacker))| {
            if let Some(attacker_ent) = attacker.entity() {
                if targets.contains(attacker_ent) {
                    *enemy = Some(*attacker_ent).into();
                }
            }
        });

    // Если у нас всё ещё нет вражины, выбираем кого попало
    world
        .query_mut::<&mut Enemy>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(doggy, enemy)| {
            if enemy.is_none() {
                *enemy = targets
                    .iter()
                    .filter(|target| **target != doggy)
                    .nth(rng.gen_range(0..targets.len() - 1))
                    .cloned()
                    .into();
            };
        });
}

/// Выбираем cтепень урона
fn randomize_damage(world: &mut World, rng: &mut impl Rng) {
    world
        .query_mut::<&mut Damage>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(_, damage)| {
            *damage = rng.gen_range(1..=8).into();
        })
}

/// Лаем
fn bark(world: &World) {
    world
        .query::<(&Name, &Action, &Health)>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(_, (name, action, health))| match action {
            Action::Barks => println!("{}[{}] barks.", name, health),
            _ => (),
        })
}

/// Рычим
fn snarls(world: &World) {
    world
        .query::<(&Name, &Action, &Health)>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(_, (name, action, health))| match action {
            Action::Snarls => println!("{}[{}] snarls.", name, health),
            _ => (),
        })
}

/// Кусаем
fn attack(world: &mut World) {
    // Сохраняем все найденные атаки как (атакующий, атакуемый)
    let mut target_entities: Vec<Entity> = vec![];
    let mut target_attackers: Vec<Entity> = vec![];
    let mut target_damages: Vec<u32> = vec![];

    // Находим кого и на сколько повредила каждая собака
    world
        .query::<(&Action, &Enemy, &Damage)>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(doggy, (action, enemy, damage))| {
            if let (Action::Attack, Some(enemy)) = (action, enemy.entity()) {
                target_entities.push(*enemy);
                target_attackers.push(doggy);
                target_damages.push(damage.value());
            }
        });

    // Пишем что случилось в консоль
    let mut target_names_healths: HashMap<Entity, (String, Health)> = HashMap::default();

    world
        .query::<(&Health, &Name)>()
        .with::<Alive>()
        .into_iter()
        .filter(|(doggy, _)| target_entities.contains(doggy))
        .for_each(|(doggy, (health, name))| {
            target_names_healths.insert(doggy, (name.to_string(), *health));
        });

    world
        .query::<(&Action, &Name, &Health, &Enemy, &Damage)>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(_, (action, name, health, enemy, damage))| {
            if let (Action::Attack, Some(enemy_ent)) = (action, enemy.entity()) {
                if let Some((target_name, target_health)) = target_names_healths.get(&enemy_ent) {
                    println!(
                        "{}[{}] attacks {}[{}] for {} damage",
                        name, health, target_name, target_health, damage
                    )
                }
            }
        });

    // Применяем урон к собакам
    world
        .query_mut::<(&mut Health, &mut Attacker)>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(doggy, (health, attacker))| {
            target_entities
                .iter()
                .zip(target_damages.iter())
                .zip(target_attackers.iter())
                .filter(|((wounded, _), _)| **wounded == doggy)
                .for_each(|((_, damage), new_attacker)| {
                    *health = health.value().saturating_sub(*damage).into();
                    *attacker = Some(*new_attacker).into();
                });
        });
}

/// Подыхаем
fn death(world: &mut World) {
    let mut dead_dogs = vec![];
    world
        .query::<(&Health, &Name)>()
        .with::<Alive>()
        .into_iter()
        .for_each(|(doggy, (health, name))| {
            if health.value() == 0 {
                println!("Sadly {} is died", name);
                dead_dogs.push(doggy)
            }
        });

    dead_dogs.into_iter().for_each(|doggy| {
        world.remove_one::<Alive>(doggy).unwrap();
    });
}

// fn debug_print(world: &World) {
//     world.query::<(Option<&Alive>, &Name, &Health, &Action, &Attacker, &Enemy, &Damage)>().into_iter().for_each(
//         |(entity, (alive, name, health, action, attacker, enemy, damage))| {
//             println!(
//                 "entity: {:?}, alive: {:?}, name: {:?}, health: {:?}, action: {:?}, attacker: {:?}, enemy: {:?}, damage: {:?}",
//                 entity, alive, name, health, action, attacker, enemy, damage
//             );
//         }
//                                                                                                  );
// }

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Street fight")]
struct Opt {
    /// Number of turns
    #[structopt(short, long, default_value = "5")]
    turns: u32,

    #[structopt(short, long)]
    /// Dog name (could be passed many times)
    dog_names: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();

    // Количество ходов
    let n_turns = opt.turns;

    // Тут можно добавить сколько угодно имён собак, для каждого имени будет создана собака
    let mut names: Vec<&str> = if opt.dog_names.is_empty() {
        vec![&"Rex", &"Fluffy"]
    } else {
        opt.dog_names.iter().map(|s| s.as_str()).collect()
    };

    let mut world = World::default();
    populate_world(&mut world, &names);

    // Генератор случайных чисел
    let mut rng = thread_rng();

    println!("Street fight begins!");
    for n in 1..=n_turns {
        println!("Turn {}", n);

        // Запускаем все системы
        choose_enemy(&mut world, &mut rng);
        randomize_damage(&mut world, &mut rng);
        choose_action(&mut world, &mut rng);
        bark(&mut world);
        snarls(&mut world);
        attack(&mut world);
        death(&mut world);
        // debug_print(&mut world);

        if let Some((0, (_, name))) = world
            .query::<&Name>()
            .with::<Alive>()
            .iter()
            .enumerate()
            .last()
        {
            println! {"Only one dog is alive! The winner is {}", name};
            break;
        }
    }
}
