<ol>
   <li>
      <a href="#what-is-eportal">About the project</a>
      <ul>
         <li><a href="#what-is-eportal">What is eportal?</a></li>
      </ul>
      <ul>
         <li><a href="#plants">Plants</a></li>
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
      <ul><li><a href="#how-to-download">How to download?</a></li></ul>
      <ul>
         <ul>
            <li><a href="#compile-from-source">Compile from source</a></li>
         </ul>
         <ul>
            <li><a href="#get-a-linux-or-windows-binary-from-the-releases">Get a Linux or Windows binary from the releases</a></li>
         </ul>
      </ul>
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
![image](https://github.com/user-attachments/assets/fd844fd7-512e-4f3e-b683-105ef0fd8e34)

## What is eportal?
`e` in `eportal` stands for `evolution`, so eportal is an evolution portal where you can create your own configurable worlds.

## Plants
Plants spawn in different places and eventually die. They serve as food for the bodies.
The plants are split into green and yellow: grass and bananas. Bananas give more energy than grass.

## Bodies
Bodies are split into types. Each type has a unique color.

Food can be:
- [plants](#plants)
- [living bodies of other types](#bodies)
- [dead](#death) bodies (crosses)


Every body has an eating strategy that can be:
- `Herbivorous`: eats [plants](#plants)
- `Carnivorous`: eats [living and dead bodies](#bodies)
- `Omnivorous`: eats everything

When the body reaches food, the food disappears, which means it's been eaten by the body. The body gets [energy](#energy) from it.

When the body goes beyond the evolution field, it gets teleported to the opposite border.

### Energy
The body gets energy by eating food:
- `plants`: Plants of one kind all give the same energy.
- `living bodies`: Living bodies give their current energy.
- `dead bodies`: Dead bodies give the energy they had the moment they died.

[Omnivorous bodies](#bodies) only get certain part the of energy of the food they eat, unlike carnivorous
and herbivorous bodies.

The body constantly spends energy on:
- `living`
- `movement`
- `skills`
- `healing from viruses`
- `vision`: Carnivorous bodies spend energy on vision only when someone is within their vision distance.

### Properties
The body has properties that affect whether it survives.

- `energy`: The amount of energy the body has left. When the energy goes below a specific point, the body [dies](#death).
- `speed`: The speed the body moves with.
- `vision distance`: The radius of how far the body can see.
- `division threshold`: The threshold of energy the body has to be eligible to [procreate](#procreation).
- `skills`: The [skills](#skills) the body has.
- `viruses`: The [viruses](#viruses) the body has been infected with.
- `lifespan`: How long the body has [left](#death) to live in case it theoretically stands still. The life shortens when the body moves, depending on the speed.

### Procreation
The body procreates and therefore splits into 2 other bodies if and only if:
- It isn't being chased by anyone
- Its energy is greater than its division threshold


Both children of the body get the following properties directly:
- eating strategy
- viruses
- energy / 2

Both children of the body get the following properties with a deviation:
- speed
- vision distance
- division threshold
- skills

### Death
The body dies if:
- Its energy drops below a specific point
- Its lifetime is over
- It's been eaten

It becomes an eatable cross in the first 2 cases. The cross disappears in a certain period of time.

### Skills
Every body can coincidentally get or lose any of the following skills:

0. `Do not compete with relatives`: When the body sees another body of its type following a plant, body, or cross, it doesn't try to follow the same object.
1. `Do not compete with younger relatives`: When the body sees a younger body of its type following a plant, body, or cross, it doesn't try to follow the same object.
2. `Alive when arrived`: When the body sees a plant or dead body, the body makes sure it doesn't die before it gets to the food.
3. `Profitable when arrived`: When the body sees a living body, cross, or plant, the body makes sure it'll get more energy when it eats it than it spends on getting to that food.
4. `Prioritize faster chasers`: When the body sees it's being chased by several other bodies, it escapes from the faster one.
5. `Avoid new viruses`: When the body sees a living body, it makes sure that eating it won't result in getting viruses the body doesn't have yet.
6. `Will arrive first`: When the body sees there are other bodies following the same food as it is following, the body makes sure it gets there the fastest.
7. `Eat crosses of my type`: When the body sees a cross of its type, it eats the cross to make sure bodies of other types don't eat it and therefore don't get the energy.
8. `Avoid infected crosses`: When the body sees a cross contains viruses the body itself doesn't have yet, it avoids the cross.

### Viruses
Every omnivorous or carnivorous body can be infected (a red dot is shown on top of the body) with the following viruses:
1. `Speed virus`: The virus steals specific part of the body's speed the moment the body gets infected with the virus.
2. `Vision virus`: The virus steals specific part of the body's vision distance the moment the body gets infected with the virus.

The body gets infected with a virus:
- At the start of the evolution
- Throughout the evolution by eating infected living or dead bodies

The body spends energy on healing from the virus. However, if the body has got rid of the virus, the effects
of the virus stay on.

## Conditions
The evolution process is periodically struck by conditions.
- `Drought`: Fewer plants grow.
- `Rain`: More plants grow.

## Zoom
When the zoom mode is on, you can see:
- Vision distance circles
- Lines between the body and its food
- The info about each specific body

## Interactions
- <kbd>esc</kbd>: quit the program
- <kbd>left mouse button</kbd>: toggle the [zoom](#zoom) mode
- <kbd>1</kbd>: toggle showing the info
- <kbd>2</kbd>: toggle showing the info about the current evolution
- <kbd>3</kbd>: update the mid-evolution settings from the config if there have been changes
- <kbd>space</kbd>: toggle drawing

## Configuration
All configuration is done through `config.toml`.
The config has to:
- be named correctly
- be in the same directory as the binary/executable
- have valid syntax

The default config is here: https://github.com/kul-sudo/eportal/blob/main/config.toml

## How to run?
### Compile from source
If you don't have the Rust compiler installed yet, get it from `rustup.rs`.
Then simply run this command wherever you feel comfortable:
```sh
git clone https://github.com/kul-sudo/eportal; cd eportal; RUSTFLAGS='-C target-cpu=native' cargo run --release;
```

### Get a Linux or Windows binary from the releases
https://github.com/kul-sudo/eportal/releases


# How to help the project?
## Financially
Even small donations are appreciated: https://paypal.me/rustprogramming

## Contributions
Especially:
1. Better organizing the code
2. Documentantion in the code and in the README (what you're reading right now)
3. Adding more viruses and skills
4. Handle values in `config.toml` leading to crashes
