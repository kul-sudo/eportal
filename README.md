<ol>
   <li>
      <a href="#what-is-eportal">About the project</a>
      <ul>
         <li><a href="#what-is-eportal">What is eportal?</a></li>
      </ul>
      <ul>
         <li><a href="#bodies">Plants</a></li>
         <li><a href="#bodies">Bodies</a></li>
         <ul>
            <li><a href="#energy">Energy</a></li>
         </ul>
         <ul>
            <li><a href="#properties">Properties</a></li>
         </ul>
         <ul>
            <li><a href="#procreation">Procreation</a></li>
         </ul>
         <ul>
            <li><a href="#death">Death</a></li>
         </ul>
         <ul>
            <li><a href="#skills">Skills</a></li>
         </ul>
         <ul>
            <li><a href="#viruses">Viruses</a></li>
         </ul>
      </ul>
      <ul><li><a href="#zoom">Zoom</a></li></ul>
      <ul><li><a href="#interactions">Interactions</a></li></ul>
      <ul><li><a href="#configuration">Configuration</a></li></ul>
   </li>
   <li><a href="#how-to-help-the-project">How to help the project?</a></li>
   <ul>
      <li><a href="#financially">Financially</a></li>
   </ul>
   <ul>
      <li><a href="#contributions">Contributions</a></li>
   </ul>
</ol>

  
# About the project
## What is eportal?
`e` in `eportal` stands for `evolution`, so eportal is a configurable evolution portal where you can create your own worlds.

## Plants
Plants (the green triangles) spawn in different places and eventually die. They're food for the bodies.

## Bodies
Bodies are split into types. Each type has a unique color.


Every body has an eating strategy that can be:
- `Passive (circles)`: When the body sees no food, it stands still, waiting for it.
- `Active (rectangles)`: When the body sees no food, it starts walking in different directions, hoping to find it.

Food can be:
- [plants](#plants)
- [living bodies of other types](#bodies)
- [dead](#death) bodies (crosses)

When a body reaches food, it disappears, which means it's been eaten by the body. The body gets [energy](#energy) from it.

### Energy
The body gets energy by eating food:
- `plants`: Plants all give the same energy.
- `living bodies`: Living bodies give their current energy.
- `dead bodies`: Living bodies give the energy they had the moment they died.

The body constantly spends energy on:
- living
- movement
- vision
- skills
- healing from viruses

### Properties
The body has properties that affect whether it survives.

- `energy`: The amount of energy the body has left. When the energy goes below a specific point, the body [dies](#death).
- `speed`: The speed the body moves with.
- `vision distance`: The radius of how far the body can see.
- `eating strategy`: The body can be either `passive` or `active`. If the body is `passive`, when it sees no food, it waits until it sees it, while if it's `active`, it walks, hoping to find something.
- `division threshold`: The threshold of energy the body has to each to eligible to [procreate](#procreation).
- `skills`: The skills the body has. Refer to [this](#skills).
- `viruses`: The viruses the body has been infected with. Refer to [this](#viruses).
- `lifespan`: How long the body has [left](#death) to live in case it theoretically stands still. The life shortens when the body moves, depending on the speed.

### Procreation <a id="procreation"></a>
The body procreates and therefore splits into 2 other bodies if and only if:
- It isn't being chased by anyone
- Its energy is greater than its division threshold


Both children of a body get the following properties with a deviation:
- speed
- vision distance
- division threshold
- skills

Both children of a body get the following properties directly:
- eating strategy
- viruses

And they get a half of their parent's energy.

### Death <a id="death"></a>
The body dies if and only if:
1. Its energy drops below a specific point
2. Its lifetime is over
3. It's been eaten

It becomes an eatable cross in the first 2 cases.

### Skills <a id="skills"></a>
Every body can coincidentally get any of the following skills:
- `Do not compete with relatives`: When the body sees another body of its type is following a plant, body, or cross, it doesn't try to do it too.
- `Alive when arrived`: When the body sees a plant or a dead body, it makes sure it doesn't die before it gets to it.
- `Profitable when arrived`: When the body sees a living body, cross, or plant, it makes sure it'll get more energy when it eats it than it spends on getting to that food.
- `Prioritize faster chasers`: When the body sees it's being chased by several other bodies, it escapes from the faster one.
- `Avoid new viruses`: When the body sees a living body, it makes sure that eating it won't result in getting viruses the body doesn't have yet.
- `Will arrive first`: When the body sees there are other bodies following the same food as it is following, it makes sure it gets there the fastest.
- `Eat crosses of my type`: When the body sees a cross of its type, it eats it to make sure bodies of other types don't eat it and therefore don't get energy from it.
- `Avoid infected crosses`: When the body sees a cross contains viruses the body itself doesn't have yet, it avoids the cross.

### Viruses <a id="viruses"></a>
Every body can be infected (a red dot is shown on top of the body) with the following viruses:
- `Speed virus`: The virus steals a specific part of the body's speed the moment the body gets infected with the virus.
- `Vision virus`: The virus steals a specific part of the body's vision distance the moment the body gets infected with the virus.

The body gets infected with a virus:
- At the start of the evolution
- Throughout the evolution by eating infected living or dead bodies.

The body spends energy on healing from the virus. However, if the body has got rid of the virus, the effects
of the virus stay on.

## Zoom
When the zoom mode is on, you can see:
- Vision distance circles
- Lines between the body and its food
- The info about each specific body

## Interactions
- <kbd>Esc</kbd>: quit the program
- <kbd>Left Mouse Button</kbd>: toggle [zoom](#zoom) mode
- <kbd>1</kbd>: toggle showing the info
- <kbd>Space</kbd>: toggle drawing

## Configuration
All configuration is done through `config.toml`.
The config has to:
- have valid syntax
- be in the same directory as the binary/executable
- be named correctly

The default config is:
```toml
# The comment after each value is the value recommended by the developers.

[body]
# For the 1st generation
bodies_n = 800 # 800 (How many bodies are spawned on the field)
passive_chance = 0.3 # 0.3 (The chance a body becomes passive)
average_energy = 1500.0 # 1500.0
average_speed = 1.5 # 1.5
average_division_threshold = 2300.0 # 2300.0
average_vision_distance = 100.0 # 100.0

# All the time (1st generation included)
skills_change_chance = 0.14 # 0.14 (The chance a body's child either gets or loses a skill)
deviation = 0.1 # 0.1 (The deviation a body gets its properties with)
lifespan = 240.0 # 240.0 (Lifespan in seconds if a body theoretically doesn't move at all)
min_energy = 1000.0 # 1000.0 (The minimum energy a body can live with)
cross_lifespan = 35 # 35 (How long a cross stays on in seconds)
const_for_lifespan = 0.000002 # 0.000002 (Makes the life of a body shorter if it moves)

[plants]
plants_density = 0.00026 # 0.00026 (The number of plants per unit area for the initial spawning)
plant_spawn_chance = 0.0000001 # 0.0000001 (The chance for a plant to be spawned per unit area)
plant_die_chance = 0.0004 # 0.0004 (The chance for a plant to die)

[energy]
energy_spent_const_for_mass = 0.00005 # 0.00005 (Part of energy constantly spent on mass)
energy_spent_const_for_skills = 0.04 # 0.04 (Part of energy constantly spent on one skill)
energy_spent_const_for_vision_distance = 0.00005 # 0.00005 (Part of energy constantly spent on vision distancce)
energy_spent_const_for_movement = 0.0004 # 0.0004 (Part of energy constantly spent on movement depending on the speed)

[viruses]
# first_generation_infection_chance: the chance the 1st generation gets infected with the virus
# speed_decrease: part of speed the virus steals
# energy_spent_for_healing: part of energy spent on healing from the virus
# heal_energy: how much energy needs to be spent to get rid of the virus

speedvirus_first_generation_infection_chance = 0.12 # 0.12
speedvirus_speed_decrease = 0.7 # 0.7
speedvirus_energy_spent_for_healing = 0.02 # 0.02
speedvirus_heal_energy = 500.0 # 500.0

visionvirus_first_generation_infection_chance = 0.1 # 0.1
visionvirus_vision_distance_decrease = 0.7 # 0.7
visionvirus_energy_spent_for_healing = 0.02 # 0.02
visionvirus_heal_energy = 500.0 # 500.0

[ui]
body_info_font_size = 17 # 17 (The font size of the info displayed over the bodies)

# Properties of a body to show over it
show_energy = false # The current energy
show_division_threshold = false # The energy a body has to reach to be able to procreate
show_body_type = false # The type ID the body is part of (can also be distinguished using colors)
show_lifespan = false # How many seconds a body has left to live if it theoretically doesn't move at all

show_skills = true # The skills of a body
# DoNotCompeteWithRelatives = 0
# AliveWhenArrived = 1
# ProfitableWhenArrived = 2
# PrioritizeFasterChasers = 3
# AvoidNewViruses = 4
# WillArriveFirst = 5
# EatCrossesOfMyType = 6
# AvoidInfectedCrosses = 7

show_viruses = true # The viruses a body has been infected with
# SpeedVirus = 0
# VisionVirus = 1
```

# How to help the project?
## Financially
Even small donations are appreciated: https://paypal.me/rustprogramming

## Contributions
Especially:
1. Better organizing the project
2. Implementing cells for bodies
3. Documentantion in the code and in the README (what you're reading right now)
4. Adding more viruses and skills
5. General optimizations
6. More explicit errors
7. On-screen info
8. Handle invalid values in `config.toml`
