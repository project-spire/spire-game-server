#pragma once

#include <spire/core/units.hpp>

namespace spire {
struct Character {
    u64 id;
};

struct Monster {
    u64 id;
};

struct Health {
    u32 max_value;
    u32 value;
};

struct Mana {
    u32 max_value;
    u32 value;
};

struct Stamina {
    u32 max_value;
    u32 value;
};

struct Shield {
    u32 value;
    u32 id;
};

struct CoreStats {
    u32 strength;
    u32 magic;
    u32 agility;
    u32 endurance;
    u32 constitution;
};
}