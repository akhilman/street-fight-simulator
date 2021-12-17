use legion::{systems::CommandBuffer, world::SubWorld, *};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};
use std::{
    collections::HashMap,
    default::Default,
    fmt,
    ops::{Deref, DerefMut},
};

// Новый тип для действия собакаки

#[derive(Clone, Copy, Debug)]
enum Action {
    Idle,
    Barks,
    Snarls,
    Attack,
}

impl Action {
    fn random(rng: &mut impl Rng, has_enemy: bool) -> Self {
        if has_enemy {
            [Self::Barks, Self::Snarls, Self::Attack]
                .choose(rng)
                .cloned()
                .unwrap()
        } else {
            Self::Idle
        }
    }
}

// Иарке для живых собакак
#[derive(Debug)]
struct Alive;

// Новый тип для последней атаковавшей сабокаки

#[derive(Default, Debug)]
struct Attacker(Option<Entity>);

impl Deref for Attacker {
    type Target = Option<Entity>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Attacker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Новый тип для урона от зубов

#[derive(Debug)]
struct Damage(u32);

impl Default for Damage {
    fn default() -> Self {
        Damage(0)
    }
}

impl Deref for Damage {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Damage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Damage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Новый тип для врага собакаки

#[derive(Default, Debug)]
struct Enemy(Option<Entity>);

impl Deref for Enemy {
    type Target = Option<Entity>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Enemy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Новый тип для жизни собакаки

#[derive(Clone, Copy, Debug)]
struct Health(u32);

impl Default for Health {
    fn default() -> Self {
        Health(10u32)
    }
}

impl Deref for Health {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Health {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Health {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Новый тип для имени собакаки

#[derive(Debug)]
struct Name(String);

impl Name {
    fn new<T: ToString>(name: T) -> Self {
        Name(name.to_string())
    }
}

impl Deref for Name {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Name {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

/// Заполняем мир собакаками
fn populate_world<I, T>(world: &mut World, names: I)
where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
{
    // Для каждой собакаки создаётся сущность со всеми необходимыми компонентами
    let _dogs: Vec<Entity> = names
        .into_iter()
        .map(|name| -> Entity {
            world.push((
                Alive,
                Name::new(name.as_ref()),
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
#[system(for_each)]
#[filter(component::<Alive>())]
fn choose_action(action: &mut Action, enemy: &Enemy, #[resource] rng: &mut ThreadRng) {
    *action = Action::random(rng, enemy.is_some())
}

/// Выбираем врагов
#[system]
#[write_component(Enemy)]
#[read_component(Entity)]
#[read_component(Attacker)]
fn choose_enemy(world: &mut SubWorld, #[resource] rng: &mut ThreadRng) {
    // Собираем всех кого можно атактовать
    let targets: Vec<Entity> = <Entity>::query()
        .filter(component::<Alive>())
        .iter_mut(world)
        .cloned()
        .collect();

    // Не атакуем дохлых собакак
    <&mut Enemy>::query()
        .filter(component::<Alive>())
        .iter_mut(world)
        .for_each(|enemy| {
            if let Enemy(Some(enemy_ent)) = enemy {
                if !targets.contains(enemy_ent) {
                    *enemy = Enemy(None);
                }
            }
        });

    // Теперь выставляем целью ту псину, которая нас покусала
    <(&mut Enemy, &Attacker)>::query()
        .filter(component::<Alive>())
        .iter_mut(world)
        .for_each(|(enemy, attacker)| {
            if let Attacker(Some(attacker_ent)) = attacker {
                if targets.contains(attacker_ent) {
                    *enemy = Enemy(Some(attacker_ent.clone()));
                }
            }
        });

    // Если у нас всё ещё нет вражины, выбираем кого попало
    <(Entity, &mut Enemy)>::query()
        .filter(component::<Alive>())
        .iter_mut(world)
        .for_each(|(doggy, enemy)| {
            if enemy.is_none() {
                *enemy = Enemy(
                    targets
                        .iter()
                        .filter(|target| **target != *doggy)
                        .nth(rng.gen_range(0..targets.len() - 1))
                        .cloned(),
                );
            };
        });
}

/// Выбираем cтепень урона
#[system(for_each)]
#[filter(component::<Alive>())]
fn randomize_damage(damage: &mut Damage, #[resource] rng: &mut ThreadRng) {
    **damage = rng.gen_range(1..=8);
}

/// Лаем
#[system(for_each)]
#[filter(component::<Alive>())]
fn bark(name: &Name, action: &Action, health: &Health) {
    match action {
        Action::Barks => println!("{}[{}] barks.", name, health),
        _ => (),
    }
}

/// Рычим
#[system(for_each)]
#[filter(component::<Alive>())]
fn snarls(name: &Name, action: &Action, health: &Health) {
    match action {
        Action::Snarls => println!("{}[{}] snarls.", name, health),
        _ => (),
    }
}

/// Кусаем
#[system]
#[read_component(Entity)]
#[write_component(Health)]
#[write_component(Attacker)]
#[read_component(Name)]
#[read_component(Action)]
#[read_component(Enemy)]
#[read_component(Damage)]
fn attack(world: &mut SubWorld) {
    // Сохраняем все найденные атаки как (атакующий, атакуемый)
    let mut target_entities: Vec<Entity> = vec![];
    let mut target_attackers: Vec<Entity> = vec![];
    let mut target_damages: Vec<u32> = vec![];

    // Находим кого и на сколько повредила каждая собака
    <(Entity, &Action, &Enemy, &Damage)>::query()
        .filter(component::<Alive>())
        .iter_mut(world)
        .for_each(|(doggy, action, enemy, damage)| {
            if let (Action::Attack, Some(enemy)) = (action, enemy.deref()) {
                target_entities.push(*enemy);
                target_attackers.push(*doggy);
                target_damages.push(**damage);
            }
        });

    // Записываем имена и здоровье тех кого покусали в словарь
    let mut target_names_healths: HashMap<Entity, (String, Health)> = HashMap::default();

    <(Entity, &Health, &Name)>::query()
        .filter(component::<Alive>())
        .iter_mut(world)
        .filter(|(doggy, _, _)| target_entities.contains(doggy))
        .for_each(|(doggy, health, name)| {
            target_names_healths.insert(*doggy, (name.to_string(), *health));
        });

    // Пишем что случилось в консоль
    <(&Action, &Name, &Health, &Enemy, &Damage)>::query()
        .filter(component::<Alive>())
        .iter_mut(world)
        .for_each(|(action, name, health, enemy, damage)| {
            if let (Action::Attack, Enemy(Some(enemy_ent))) = (action, enemy) {
                if let Some((target_name, target_health)) = target_names_healths.get(&enemy_ent) {
                    println!(
                        "{}[{}] attacks {}[{}] for {} damage",
                        name, health, target_name, target_health, damage
                    )
                }
            }
        });

    // Применяем урон к собакакам
    <(Entity, &mut Health, &mut Attacker)>::query()
        .filter(component::<Alive>())
        .iter_mut(world)
        .for_each(|(doggy, health, attacker)| {
            target_entities
                .iter()
                .zip(target_damages.iter())
                .zip(target_attackers.iter())
                .filter(|((wounded, _), _)| **wounded == *doggy)
                .for_each(|((_, damage), new_attacker)| {
                    **health = health.saturating_sub(*damage);
                    **attacker = Some(*new_attacker);
                });
        });
}

/// Подыхаем
#[system(for_each)]
#[filter(component::<Alive>())]
fn death(entity: &Entity, health: &Health, name: &Name, commands: &mut CommandBuffer) {
    if **health == 0 {
        println!("Sadly {} is dead", name);
        commands.remove_component::<Alive>(*entity);
    }
}

// #[system(for_each)]
// fn debug_print(
//     entity: &Entity,
//     alive: Option<&Alive>,
//     name: &Name,
//     health: &Health,
//     action: &Action,
//     attacker: &Attacker,
//     enemy: &Enemy,
//     damage: &Damage,
// ) {
//     println!(
//         "entity: {:?}, alive: {:?}, name: {:?}, health: {:?}, action: {:?}, attacker: {:?}, enemy: {:?}, damage: {:?}",
//         entity, alive, name, health, action, attacker, enemy, damage
//     );
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

    // Тут можно добавить сколько угодно имён собак, для каждого имени будет создана собакака
    let mut names: Vec<&str> = if opt.dog_names.is_empty() {
        vec![&"Rex", &"Fluffy"]
    } else {
        opt.dog_names.iter().map(|s| s.as_str()).collect()
    };

    let mut world = World::default();

    // Генератор случайных чисел
    let mut rng = thread_rng();

    // Порядок ходов собак будет случайным
    names.as_mut_slice().shuffle(&mut rng);

    populate_world(&mut world, &names);

    // Добавляем системы в планеровщик
    let mut schedule = Schedule::builder()
        // Системы использующие рандом будут выполняться в главном триде
        .add_thread_local(choose_enemy_system())
        .add_thread_local(randomize_damage_system())
        .add_thread_local(choose_action_system())
        // Остальные системы могут быть распараллелены
        .add_system(bark_system())
        .add_system(snarls_system())
        .add_system(attack_system())
        .add_system(death_system())
        // .add_thread_local(debug_print_system())
        .flush()
        .build();

    // Генератор случайнх чисел будет глобальным ресурсом
    let mut resources = Resources::default();
    resources.insert(rng);

    println!("Street fight begins!");
    for n in 1..=n_turns {
        println!("Turn {}", n);
        schedule.execute(&mut world, &mut resources);
        if let Some((0, name)) = <&Name>::query()
            .filter(component::<Alive>())
            .iter(&world)
            .enumerate()
            .last()
        {
            println! {"Only one dog is alive! The winner is {}", name};
            break;
        }
    }
}
