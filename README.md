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
![image](https://github.com/kul-sudo/eportal/assets/95244851/c3791374-d301-487d-9f3b-40d410e23026)

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
- `dead bodies`: Dead bodies give the energy they had the moment they died.

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
- `division threshold`: The threshold of energy the body has to be eligible to [procreate](#procreation).
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
The body dies if:
1. Its energy drops below a specific point
2. Its lifetime is over
3. It's been eaten

It becomes an eatable cross in the first 2 cases.

### Skills <a id="skills"></a>
Every body can coincidentally get any of the following skills:
- `Do not compete with relatives`: When the body sees another body of its type is following a plant, body, or cross, it doesn't try to follow the same object.
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
- <kbd>esc</kbd>: quit the program
- <kbd>Left Mouse Button</kbd>: toggle [zoom](#zoom) mode
- <kbd>1</kbd>: toggle showing the info
- <kbd>space</kbd>: toggle drawing

## Configuration
All configuration is done through `config.toml`.
The config has to:
- have valid syntax
- be in the same directory as the binary/executable
- be named correctly

The default config is here: https://github.com/kul-sudo/eportal/blob/main/config.toml

## How to download?
### Compile from source
If you don't hate the Rust compiler installed yet, get it from `rustup.rs`.
Then simply run this command wherever you feel comfortable:
```sh
git clone https://github.com/kul-sudo/eportal; cd eportal; cargo run --release;
```

### Get a Linux or Windows binary from the releases
https://github.com/kul-sudo/eportal/releases


# How to help the project?
## Financially
Even small donations are appreciated: https://paypal.me/rustprogramming

## Contributions
Especially:
1. Better organizing the code
2. Implementing cells for bodies
3. Documentantion in the code and in the README (what you're reading right now)
4. Adding more viruses and skills
5. General optimizations
6. More explicit errors
7. On-screen info
8. Handle invalid values in `config.toml`
