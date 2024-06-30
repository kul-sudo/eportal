# eportal
## What's eportal?
`e` in `eportal` stands for `evolution`, so eportal is a configurable evolution portal. When the evolution process starts, bodies start walking around the field looking for plants and bodies of other kinds for food.

## For rookies: how does evolution work?
Let's say a creature has 2 children. The 1st child for a certain reason has a slightly higher intelligence, and the 2nd one runs faster and also has a slightly higher intelligence.
Then it turns out running fast *and* thinking better requires a bit too much energy to be spent, so the 1st one survives instead of the 2nd one.
\
Then the children of the survived creature have other deviated properties, and it goes on and on. And this brute-force way results in constant perfection of those properties.

## Bodies
Bodies are split into types. Each type has a unique color.
They walk around the map looking for:
1. `Plants`: green triangle all giving the same energy
2. `Bodies of other types`: bodies of one type don't eat each other.

### Properties
1. `energy`: The amount of energy the body has left. When the energy goes below a specific point, the body [death](#death).
2. `speed`: The speed the body moves with. The higher it is, the more energy is spent on it.
3. `vision distance`: The radius of how far the body can see. The higher it is, the more energy is spent on it.
4. `eating strategy`: The body can be either `passive` or `active`. If the body is `passive`, when it sees no food, it waits until it sees it, while if it's `active`, it walks hoping to find something.
5. `division threshold`: The threshold of energy the body has to each to eligible to [procreate](#procreation).
6. `skills`: The skills the body has. Refer to [this](#skills). The energy spent on the skills is the number of skills multiplied by a specific `k`.
7. `viruses`: The viruses the body has been infected with. Refer to [this](#viruses).
8. `lifespan`: How long the body has [left](#death) to live in case it theoretically stands still. The life shortens when the body moves, depending on the speed.

### Procreation <a id="procreation"></a>
The body procreates and therefore splits into 2 other bodies if and only if:
1. It isn't being chased by anyone
2. Its energy is greater than its division threshold

Both children of a body get the following properties with a deviation:
1. speed
2. vision distance
3. division threshold

And they get a half of their parent's energy.

### Death <a id="death"></a>
The body dies only and only if:
1. If energy drops below a specific point
2. If its lifetime is over
3. If it's been eaten

It becomes an eatable cross in the first 2 cases.

### Skills <a id="skills"></a>
Every body can coincidentally get any of the following skills:
1. `Do not compete with relatives`: When the body sees another body of its type is following a plant or body of anothet type, it doesn't try to do it too.
2. `Alive when arrived`: When the body sees a plant or a dead body, it makes sure it doesn't die before it gets to it.
3. `Profitable when arrived`: When athebody sees a living body, cross, or plant, it makes sure it'll get more energy than it spends on getting to that food.
4. `Prioritize faster chasers`: When the body sees it's being chased by several other bodies, it escaped from the faster one.
5. `Avoid new viruses`: When the body sees a living body or a cross, it makes sure that eating it won't result in geting viruses the body doesn't have yet.
6. `Will arrive first`: When the body sees there are other bodies following the same food as it is following, it makes sure it gets there the fastest.
7. `Eat crosses of my type`: When the body sees a cross of its type, it eats it to make sure bodies of other types don't eat it and therefore don't get energy from it.
8. `Avoid infected crosses`: When the body sees a cross contains viruses the body itself doesn't have yet, it avoids the cross.

### Viruses <a id="viruses"></a>
Every body can be infected with the following viruses:
1. `Speed virus`: the virus steals a specific part of the body's speed.
2. `Vision virus`. the virus steals a specific part of the body's vision distance.

However, keep in mind the children get the original properties.
