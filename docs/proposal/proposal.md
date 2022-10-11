# School of Computing &mdash; Year 4 Project Proposal Form

## SECTION A

|                     |                          |
|---------------------|--------------------------|
|Project Title:       | Entity Relations in Bevy |
|Student 1 Name:      | Yogesvara Das            |
|Student 1 ID:        | 19712101                 |
|Student 2 Name:      | David Mc Guigan          |
|Student 2 ID:        | 19425942                 |
|Project Supervisor:  | David Sinclair           |





## SECTION B

### Introduction

Existing engine architectures in commercial game engines are stuck in time. In the status quo:
- Problems can only be expressed in a manner that is procedural, imperitive and object oriented.
- Code is not very reusable or maintainable and gets complicated.
- Scalable parallelism is difficult to achieve.

The Entity Component System (ECS) architecture is one emerging alternative technology that aims
to address many of these problems with existing engines. ECS is data oriented in philosophy but
does not have a design mantra. Different interpretations and implementations of ECS exist,
many of which have seen major use and success in games already. The most notable examples being
Overwatch and Minecraft Bedrock Edition. Additionally commercial engines are gradually offering
their own takes on ECS. Unity has its Data Oriented Tech Stack (DOTS) and Unreal has Sequencer.

A popular open source ECS engine is Bevy. Bevy is a game engine written in Rust and is the second
most starred game engine on github. It features a very ergonomic declarative API reminiscent of
structural typing or SQL via a type level DSL.


### Outline

ECS is a very bleeding edge technology and in its current state certain things are difficult to
express with pure ECS. One of the desired additions for Bevy that could address this in major
areas is Entity Relations. Entity Relations would add the capability of expressing relations
in a manner similar to Prolog.



### Background

Student 1 being a user of Bevy discovered this project idea from community interaction.
Out of a desire to work on something they're interested in and only having other project ideas
that were too ambitious for CA400 they inquired Bevy's maintainers about items in their backlog
that would make a good final year university project. Without hesitation Entity Relations was
suggested.

Inspired by another ECS that has a relations implementation (flecs), Bevy's maintainers
and contributors have an existing interest in seeing the engine dive further into the
relational paradigm. It is agreed to be an enabling technology that would complement existing
Bevy features and fit in with Bevy's take on ECS very well.

### Achievements

This project will increase the ergonomics of the Bevy game engine by adding the ability to express
Entity Relations. This will make the API even more declarative and enable its users to model
things in ways they previously were not able to. Some of the user groups that will
benefit from this addition:
- Game developers (Bevy's primary user base)
- Native App developers (Can be used to make more than just games like Unity and Unreal Engine)
- Bevy developers (Some Bevy features are defined in terms of other Bevy features)



### Justification

- This will add more options for implementing new technologies for Bevy and/or its ecosystem.
- Allows expressing patterns like shared components and hierarchies in a more ergonomic fashion.
- Enables other component, indexing and lookup strategies based around the game's unique logic
and not solely on an Entities component set. This adds both reusable optimisations/patterns and
further increases declaritiveness. Some identified examples:
    - Graphs for pathfinding.
    - Quadtree/Octree or Spatial Hashing for large scale physics.
    - Springs and Joints in colliders.
- The work done here further develops on an emerging paradigm. Relations and structural mechanisms
complement each other well. A like pairing already exists in relational databases but when paired
together with a language level mechanism for components they seem like missing halves of each other.
Discoveries made in projects like this could go on to inform the design of any new programming
languages that attempt to promote these primitives from a library status to first class status.



### Programming language(s)

- Rust



### Programming tools / Tech stack

- Bevy: The existing ECS we will be adding Entity Relations to
- Cargo: Dependency management and testing
- Rustc: Rust compiler
- Rustdoc: Docgen



### Learning Challenges

- Rust
- ECS Theory
- Bevy Codebase
- Relational Paradigm
- Library/API Design



### Breakdown of work

#### Student 1

- Relations
    - Design and implement an API to allow users to define, query and filter entity relations.
    - Optimise storage and retrieval strategies.



#### Student 2

- Relation cleanup policies (Relations introduce footguns when despawning)
    - Design and implement an API to allow users to define what clean up policies to use
    for what relational patterns when despawning entities in relational hierarchies.
    - Optimise pattern checking and removals.
