Names in the code must make sense on their own; this list only keeps each game term spelled the same way everywhere.

# Glossary

Code uses these exact words for domain concepts, spelled as shown, with no abbreviations.

## Core loop and world

| Term | Code form | Meaning |
| --- | --- | --- |
| Player | `Player` | The person playing; use for state that belongs to the player in every mode. |
| Dwarf | `Dwarf` | The top-down avatar used on the homestead, in the village, in zones and in the mine. |
| Hunter | `Hunter` | The side-scroll avatar used in hunting grounds and the dungeon. |
| Homestead | `Homestead` | The player's farm zone with the farmhouse, fields, forge yard and mine entrance. |
| Hearthspire Village | `HearthspireVillage` | The village zone with the shops, lodge, smithy and town square. |
| Misty Forest | `MistyForest` | The forest zone east of the homestead. |
| Riverlands | `Riverlands` | The river zone with the Clockwork Dam and the allotment. |
| Highland Ruins | `HighlandRuins` | The magitech city ruins zone opened in Act 2. |
| Ember Caldera | `EmberCaldera` | The volcanic zone at the edge of the ruins. |
| Floating Islets | `FloatingIslets` | The sky islands zone opened in Act 3. |
| Destiny Crater | `DestinyCrater` | The finale arena on the Floating Islets. |
| Zone | `Zone` | One top-down world area the player travels between. |
| Zone transition | `ZoneTransition` | A rect that moves the player from one zone to another. |
| Door | `Door` | An entrance into an interior. |
| Hunt gate | `HuntGate` | The entrance from a zone into its hunting ground. |
| Repair site | `RepairSite` | A quest-gated obstacle such as a broken bridge or collapsed pass. |
| Fast travel point | `FastTravelPoint` | An airship stop the player can travel to directly. |
| Airship Mast | `AirshipMast` | The village airship dock used for courier and fast travel. |
| Sky tram | `SkyTram` | The shortcut from the ruins to the islets after its bundle. |
| World flags | `world_flags` | Saved booleans that record world progress. |
| Story flag | `StoryFlag` | One named story milestone stored in `story_flags`. |
| Story beat | `StoryBeat` | A scripted story step with requirements and a trigger. |
| Cutscene | `Cutscene` | An ordered list of scripted steps played without player control. |
| Act | `Act` | One major chapter of the story. |
| Journal | `Journal` | The story log plus the collectible journal pages. |
| Letter | `Letter` | One piece of mail delivered to the mailbox. |
| Pact | `Pact` | The finale choice: Guardian, Kindred or Free. |
| Interactable | `Interactable` | A component marking something the player can interact with. |
| Energy | `tool_energy` | The homestead resource spent by tool use and restored by sleep. |
| Stamina | `Stamina` | The combat resource spent on evades, guards and charges. |

## Day and season

| Term | Code form | Meaning |
| --- | --- | --- |
| Day clock | `DayClock` | The resource holding the current time, day, season and year. |
| Day phase | `DayPhase` | The part of the day: Morning, Midday, Afternoon, Evening or Night. |
| Calendar day | `calendar_day` | The running day count since the save began. |
| Day of season | `day_of_season` | The day within the current season, from 1 to 28. |
| Season | `Season` | Spring, Summer, Fall or Winter. |
| Year | `year` | The in-game year number. |
| Weekday | `weekday` | The day of the 7-day week, Monday to Sunday. |
| Weather | `Weather` | Sunny, Cloudy, Rain, Storm, Snow, Fog or Windy. |
| Forecast | `tomorrow_forecast` | The weather predicted for the next day. |
| Calendar | `Calendar` | Data for festivals, birthdays, story events and seasonal windows. |
| Sleep outcome | `SleepOutcome` | The energy restored and penalties applied by a sleep. |
| Overnight events | `OvernightEvents` | The queue of things resolved while the player sleeps. |
| Pass out | `pass_out` | The forced sleep at 2 AM with a penalty. |
| Festival | `Festival` | A yearly village event on a fixed calendar day. |
| Weather emitter | `WeatherEmitter` | A per-zone source of rain, snow, fog or storm effects. |

## Mining

| Term | Code form | Meaning |
| --- | --- | --- |
| Hearthspire Crystal Mines | `CrystalMines` | The 40-floor top-down mine entered from the homestead. |
| Mine floor | `MineFloor`, `mine_floor` plus a number | One numbered level of the mine. |
| Stratum | `Stratum` | A band of ten floors sharing a theme, ores and monsters. |
| Upper Shafts | `UpperShafts` | Stratum of floors 1 to 10. |
| Clockwork Galleries | `ClockworkGalleries` | Stratum of floors 11 to 20. |
| Crystal Veins | `CrystalVeins` | Stratum of floors 21 to 30. |
| Molten Deep | `MoltenDeep` | Stratum of floors 31 to 40. |
| Destiny Vault | `DestinyVault` | The endless post-story floors from 41 down. |
| Rest floor | `rest_floor` | Every fifth floor, with no monsters, an elevator and a supply crate. |
| Ore node | `OreNode` | A breakable rock that drops ore, gems or stone. |
| Node kind | `NodeKind` | The type of a gathering node, such as Copper or Stormglass. |
| Ore kind | `OreKind` | The ore type a node yields. |
| Gem kind | `GemKind` | The gem type a gem node yields. |
| Geode | `GeodeKind` | A node or item that is cracked open for random contents. |
| Hardness | `hardness` | How many hits a node takes before the pickaxe power bonus. |
| Required tier | `required_tier` | The minimum pickaxe power that can damage a node. |
| Clank | `clank` | The outcome of hitting a node above the pickaxe tier: no damage, 1 energy. |
| Pickaxe | `Pickaxe` | The mining tool whose power gates node tiers. |
| Tool power | `power` | The tier number of a tool used against node requirements. |
| Ladder down | `LadderDown` | The hidden exit to the next floor. |
| Ladder up | `LadderUp` | The exit to the floor above, or home from floor 1. |
| Shaft hole | `ShaftHole` | A hazard that drops the player several floors. |
| Elevator | `Elevator` | The brass cage lift with a stop on every fifth floor. |
| Supply crate | `SupplyCrate` | A rest-floor crate with torches, bombs or food. |
| Treasure chest | `TreasureChest` | A chest with fixed loot. |
| Hazard | `Hazard` | A floor object that damages the player, such as a steam vent or lava tile. |
| Cave-in | `CaveIn` | A Molten Deep floor event where rocks fall across marked tiles. |
| Darkness | `Darkness` | The overlay that limits sight to the lantern radius. |
| Lantern | `Lantern` | Support gear that sets the light radius in Darkness. |
| Torch | `Torch` | A placed consumable that lights a floor until the player leaves. |
| Mining bomb | `MiningBomb` | A thrown bomb that breaks nodes and damages monsters. |
| Mine monster | `MineMonster` | A top-down enemy that lives on mine floors. |
| Ore drop | `OreDrop` | A world pickup collected on touch. |
| Smelter | `Smelter` | The homestead station that turns ore into bars. |
| Bar | `Bar` | A smelted metal used in forging and tool upgrades. |
| Gathering node | `GatheringNode` | A surface node outside the mine that respawns on sleep. |
| Ore Satchel | `OreSatchel` | A pouch that auto-sorts ore into separate slots. |

## Farming

| Term | Code form | Meaning |
| --- | --- | --- |
| Crop plot | `CropPlot` | One tilled tile that can hold a crop. |
| Crop | `Crop` | A planted crop growing in a plot. |
| Crop kind | `CropKind` | The species of a crop. |
| Crop quality | `CropQuality` | The quality tier of a harvested crop: Normal, Silver, Gold or Destiny. |
| Normal | `Normal` | The base quality tier. |
| Silver | `Silver` | The second quality tier. |
| Gold | `Gold` | The third quality tier. |
| Destiny | `Destiny` | The top quality tier. |
| Tending streak | `tending_streak` | Watering every day, which raises the Silver and Gold chance. |
| Fertilizer | `FertilizerKind` | A soil additive that raises quality or speeds growth. |
| Fertilizer tier | `fertilizer_tier` | Basic 1, Quality 2 or Deluxe 3. |
| Withered | `withered` | A crop killed by season change or lightning. |
| Regrow days | `regrow_days` | Days until a regrowing crop bears again. |
| Giant crop | `GiantCrop` | A 3×3 crop grown from a full block of the same crop. |
| Fruit tree | `FruitTree` | A tree that bears fruit in its season. |
| Trellis | `Trellis` | A crop support that blocks movement. |
| Debris | `Debris` | Weeds, twigs, stones, stumps, logs and boulders on a field. |
| Field | `Field` | One farm plot area on the homestead. |
| Greenhouse | `Greenhouse` | The restored glasshouse where crops grow in any season. |
| Sprinkler | `Sprinkler` | A placed object that waters nearby plots each morning. |
| Scarecrow | `Scarecrow` | A placed object that keeps crows off crops in its radius. |
| Crow | `Crow` | A pest that eats one unprotected crop. |
| Lightning rod | `LightningRod` | A placed object that catches lightning during storms. |
| Shipping bin | `ShippingBin` | The bin that sells its contents overnight. |
| Machine | `Machine` | A magitech processor that turns an input into an output over time. |
| Compost bin | `CompostBin` | A bin that turns waste into compost. |
| Hoe | `Hoe` | The tool that tills soil. |
| Watering can | `WateringCan` | The tool that waters plots. |
| Axe | `Axe` | The tool that chops stumps, logs and giant crops. |
| Scythe | `Scythe` | The tool that cuts grass and weeds at no energy cost. |
| Seed bag | `SeedBag` | The tool that sows seeds. |
| Charge level | `charge_level` | How long a tool or weapon use is held, which widens its effect. |
| Tool tier | `ToolTier` | The upgrade step of a tool. |
| Toolbar | `Toolbar` | The 8-slot bar of selectable tools. |
| Essence | `Essence` | An extracted crop product used as an infusion. |
| Potion | `Potion` | A brewed consumable from the farm chain. |
| Coating | `Coating` | An arrow treatment brewed from crops. |

## Fishing

| Term | Code form | Meaning |
| --- | --- | --- |
| Fishing spot | `FishingSpot` | A water area where the player can cast. |
| Fishing zone | `FishingZone` | The rect of castable water at a spot. |
| Water kind | `WaterKind` | The water type of a spot, such as pond, stream or lava. |
| Rod | `Rod` | The fishing tool with cast range, catch zone and slots. |
| Cast | `cast` | Throwing the line into water. |
| Bobber | `Bobber` | The float on the line, with states Flying, Floating, Nibble, Bite and Hooked. |
| Fish shadow | `FishShadow` | A visible hint of a fish in the zone. |
| Fishing minigame | `FishingMinigame` | The reeling contest with a catch zone and progress bar. |
| Catch zone | `catch_zone` | The bar the player keeps over the fish. |
| Bait | `Bait` | A consumable on the rod that changes bites. |
| Tackle | `Tackle` | A rod attachment with limited durability. |
| Crab pot | `CrabPot` | A placed trap that catches overnight with bait. |
| Fishing dock | `FishingDock` | The repairable dock at the homestead pond. |
| Legendary fish | `LegendaryFish` | A unique rare fish that needs Monster Bait. |
| Ice fishing hole | `IceFishingHole` | A winter spot cut through ice. |

## Forging and crafting

| Term | Code form | Meaning |
| --- | --- | --- |
| Destiny Forge | `DestinyForge` | The homestead anvil, furnace and workbench group where gear is made. |
| Forge station | `ForgeStation` | The forge entity with its sub-stations. |
| Forge level | `ForgeLevel` | The forge upgrade step from 0 to 5. |
| Dormant | `Dormant` | Forge level 0. |
| Copper Heart | `CopperHeart` | Forge level 1. |
| Brass Lung | `BrassLung` | Forge level 2. |
| Aether Core | `AetherCore` | Forge level 3. |
| Mythril Bellows | `MythrilBellows` | Forge level 4. |
| Destiny Crown | `DestinyCrown` | Forge level 5. |
| Anvil | `Anvil` | Forge sub-station. |
| Furnace | `Furnace` | Forge sub-station. |
| Bellows | `Bellows` | Forge sub-station. |
| Workbench | `Workbench` | The station for machines, bombs, traps and torches. |
| Jeweler's Bench | `JewelersBench` | The station for cutting gems and making and socketing decorations. |
| Kitchen | `Kitchen` | The farmhouse station for cooking. |
| Alchemy Still | `AlchemyStill` | The farm station for potions and coatings. |
| Essence Extractor | `EssenceExtractor` | The farm station that extracts essences. |
| Recipe | `Recipe` | A craft or upgrade definition with its station, gates and components. |
| Gear piece | `GearPiece` | One crafted weapon or armor item with its stats and skills. |
| Gear id | `GearId` | The id of a gear data row. |
| Rank | `Rank` | The gear and hunt tier: Low, High or Master. |
| Rarity | `rarity` | The gear rarity from 1 to 10. |
| Element | `Element` | Fire, Water, Thunder, Ice or Aether. |
| Affinity | `affinity` | The critical hit chance of a weapon. |
| Sharpness | `Sharpness` | The edge level of a melee weapon that drops with use. |
| Skill | `SkillId` | A passive bonus on gear with a level. |
| Set skill | `SetSkill` | A skill granted by wearing several pieces of one set. |
| Decoration | `Decoration` | A socketed jewel that grants a skill level. |
| Slot size | `SlotSize` | The size of a decoration slot, from 1 to 3. |
| Upgrade level | `upgrade_level` | How many times a gear piece has been upgraded. |
| Frame material | `FrameMaterial`, `frame_material` | Material role that sets base attack or defense and rank. |
| Core material | `CoreMaterial`, `core_material` | Material role that sets skills and set identity. |
| Polish material | `PolishMaterial`, `polish_material` | Material role that sets affinity, sharpness and rarity. |
| Lining | `lining` | Armor material role that sets resistances. |
| Infusion | `infusion` | Optional material role that sets element or status. |
| Destiny Binding | `Binding` | Spending Destiny Sparks to Lock, Enhance, Awaken or Rebind a piece. |
| Destiny Core | `DestinyCore` | The forge part where bindings are made. |
| Lock | `Lock` | Binding that keeps a skill through upgrades. |
| Enhance | `Enhance` | Binding that raises a skill one level above its max. |
| Awaken | `Awaken` | Binding that adds a size-3 slot. |
| Rebind | `Rebind` | Binding that moves a Lock to another skill. |
| Weapon tree | `WeaponTree` | An upgrade path of weapons within one weapon family. |
| Armor slot | `ArmorSlot` | Helm, Mail, Gauntlets, Greaves or Coil. |
| Armor set | `ArmorSet` | A family of armor pieces sharing a set skill. |
| Hearth Meal | `HearthMeal` | A pre-hunt meal that gives one daily buff. |
| Food buff | `FoodBuff` | A timed bonus from eating food. |
| Whetstone | `Whetstone` | A consumable that restores sharpness. |

## Combat and hunting

| Term | Code form | Meaning |
| --- | --- | --- |
| Hunting ground | `HuntingGround` | A side-scroll map of connected areas where large monsters are hunted. |
| Area | `area` | One screen group of a hunting ground joined by edge exits. |
| Base Camp | `BaseCamp` | Area 1 of a ground with supply box, gear box and tent. |
| Dungeon | `Dungeon` | The frozen Floor 1 side-scroll dungeon with King Slime. |
| Hunt quest | `HuntQuest` | A quest from the Quest Board with objectives and limits. |
| Hunt session | `HuntSession` | The running state of an accepted hunt. |
| Quest Board | `QuestBoard` | The board where hunt quests are accepted. |
| Faint | `faint` | Being knocked out in a hunt, which counts against the faint limit. |
| Carted out | `carted_out` | Being carried back to Base Camp after a faint. |
| Expedition | `Expedition` | A multi-day hunt with tent sleep. |
| Hunter Rank | `HunterRank` | The hunt progression rank that gates quests. |
| Weapon family | `WeaponFamily` | A weapon type with its own moveset. |
| Sword and Shield | `SwordAndShield` | Fast weapon family with bash, guard and item use while drawn. |
| Great Sword | `GreatSword` | Heavy weapon family built on charged hits. |
| Long Sword | `LongSword` | Counter weapon family built on the spirit gauge. |
| Dual Blades | `DualBlades` | Fast weapon family built on the demon gauge. |
| Hammer | `Hammer` | Blunt weapon family built on knockouts and charge. |
| Bow | `Bow` | Ranged weapon family built on charge levels and coatings. |
| Spear | `Spear` | Legacy weapon family kept but not expanded. |
| Weapon state | `WeaponState` | Per-family resource such as charge, spirit or demon gauge. |
| Spirit gauge | `spirit_gauge` | The Long Sword resource. |
| Demon gauge | `demon_gauge` | The Dual Blades resource. |
| Demon Mode | `DemonMode` | The Dual Blades state that drains stamina for stronger attacks. |
| Evade | `Evade` | A roll with invulnerability frames. |
| Guard | `Guard` | Blocking an attack with a weapon or shield. |
| Parry | `Parry` | A perfectly timed guard. |
| Item pouch | `ItemPouch` | The 24-slot pouch of hunt items. |
| Track clue | `TrackClue` | A sign left by a monster that fills the track gauge. |
| Track gauge | `TrackGauge` | Progress toward revealing a monster's location. |
| Monster part | `MonsterPart` | A part with its own HP that can break or be severed. |
| Break | `broken` | A part destroyed by damage, which gives a reward. |
| Sever | `severable` | Cutting off a part such as a tail. |
| Carve | `carve` | Harvesting materials from a defeated monster. |
| Capture | `capture` | Trapping and tranquilizing a weakened monster. |
| Trap | `Trap` | A placed Pitfall Trap or Shock Trap. |
| Environmental trap | `EnvironmentalTrap` | A ground hazard that hurts monsters. |
| Status effect | `StatusEffects` | Player conditions such as poisoned, burning, frozen, paralyzed, stunned, sleeping or slimed. |
| Status buildup | `StatusBuildup` | Accumulated status damage on a monster. |
| Hitzone | `hitzone` | A body region with its own damage values. |
| Damage type | `DamageType` | Cut, blunt or pierce. |
| Motion value | `motion_value` | The damage multiplier of one move. |
| Knockout | `knockout` | Stunning a monster with blunt head hits. |
| Companion | `Companion` | A villager who joins a hunt. |

## Monsters

| Term | Code form | Meaning |
| --- | --- | --- |
| Monster | `Monster` | A runtime monster entity. |
| Monster definition | `MonsterDefinition` | The data row for a monster species. |
| Monster class | `MonsterClass` | Small, Large or Elder. |
| Monster brain | `MonsterBrain` | The monster behavior state machine. |
| Move set | `MoveSet` | The list of moves a monster can use. |
| Move | `Move` | One monster attack with telegraph, active and recover times. |
| Telegraph | `Telegraph` | The readable windup before a move. |
| Phase | `phase` | A stage of a fight set by HP thresholds. |
| Enraged | `Enraged` | A monster state with faster, stronger moves. |
| Fatigued | `Fatigued` | A monster state of tired vulnerability. |
| Limp | `Limp` | A weakened monster fleeing to its nest. |
| Nest | `Nest` | The area a monster rests in. |
| Elemental shift | `elemental_shift` | A change of element and weakness between phases. |
| Weakness | `weakness` | An element or damage type a monster takes extra damage from. |
| Aggro | `Aggro` | Which target a monster is engaging. |
| Charger | `Charger` | Behavior archetype built on rushes and straight-line threats. |
| Caster | `Caster` | Behavior archetype built on ranged patterns and ground effects. |
| Tempered | `tempered` | A stronger post-game variant of a monster. |
| Elder | `Elder` | The top monster class. |
| Large monster | `LargeMonster` | A hunt target with parts and phases. |
| Small monster | `SmallMonster` | A minor monster that fills grounds and zones. |
| King Slime | `KingSlime` | The Floor 1 dungeon boss. |
| Bristleboar | `Bristleboar` | The first large monster and village threat. |
| Thornback Briarwolf | `ThornbackBriarwolf` | Poison forest large monster. |
| Glowmoth Matriarch | `GlowmothMatriarch` | Sleep forest large monster. |
| Tidecaller Kraken-Eel | `TidecallerKrakenEel` | Water large monster of the dam. |
| Ironhide Behemoth | `IronhideBehemoth` | Armored large monster of the dam works. |
| Gearjaw Drillwyrm | `GearjawDrillwyrm` | Burrowing large monster of the deep galleries. |
| Stormwing Gryphon | `StormwingGryphon` | Thunder flying large monster of the ruins. |
| Runeback Golem | `RunebackGolem` | Shielded large monster of the ruins. |
| Magmaw Salamander | `MagmawSalamander` | Fire large monster of the caldera. |
| Prism Leviathan Ray | `PrismLeviathanRay` | Aether large monster of the islets. |
| Aether Drake | `AetherDrake` | Elder monster with elemental shifts. |
| Elder Destiny Wyrm | `ElderDestinyWyrm` | The finale Elder monster. |

## Animals

| Term | Code form | Meaning |
| --- | --- | --- |
| Animal | `Animal` | A farm animal with friendship, mood and products. |
| Species | `species` | The kind of animal. |
| Friendship | `friendship` | Animal affection from 0 to 1000. |
| Mood | `mood` | Daily animal happiness from 0 to 255. |
| Product | `product_ready` | The item an animal gives when ready. |
| Product quality | `product_quality` | The quality tier of an animal product. |
| Building | `Building` | An animal building with a kind, level and capacity. |
| Coop | `Coop` | Building for chickens, ducks, clockwork hens and silk moths. |
| Barn | `Barn` | Building for cows, goats, sheep and steam-oxen. |
| Apiary | `Apiary` | Area for beehives. |
| Beehive | `Beehive` | A placeable hive that makes honey. |
| Pasture | `Pasture` | The fenced grazing area. |
| Stable | `Stable` | Building for the Brass Strider mount. |
| Silo | `Silo` | Building that stores hay. |
| Feed trough | `FeedTrough` | Trough that holds hay for animals. |
| Incubator | `Incubator` | Hatches an egg over days. |
| Hay | `Hay` | Animal feed cut with the scythe. |
| Pet | `Pet` | The cat or dog companion on the homestead. |
| Mount | `Mount` | The Brass Strider used to travel faster. |
| Sick | `sick` | An animal state caused by three unfed days. |

## Villagers

| Term | Code form | Meaning |
| --- | --- | --- |
| Villager | `Villager` | A named village resident. |
| Villager id | `VillagerId` | The id of a villager. |
| Heart | `hearts` | A villager friendship step from 0 to 10, each 250 points. |
| Heart event | `HeartEvent` | A cutscene unlocked at a heart threshold. |
| Gift | `Gift` | An item given to a villager. |
| Taste | `Taste` | Loved, Liked, Neutral, Disliked or Hated. |
| Schedule | `Schedule` | Where a villager is for each weekday and phase. |
| Dialogue | `Dialogue` | Lines a villager says, keyed by hearts, season and context. |
| Request | `Request` | A daily task from the request board. |
| Request board | `RequestBoard` | The town square board of daily requests. |
| Personal questline | `Questline` | A multi-step quest chain from one villager. |
| Farm helper | `FarmHelper` | A villager service that does a daily homestead chore. |
| Birthday | `birthday` | The day a villager values gifts more. |

## Progression and economy

| Term | Code form | Meaning |
| --- | --- | --- |
| Wallet | `Wallet` | The player's gold and sparks. |
| Gold | `gold` | The spendable currency. |
| Destiny Spark | `DestinySpark` | The rare currency spent on Destiny Binding and endgame unlocks. |
| Reputation | `Reputation` | Village standing points that gate upgrades and acts. |
| Reputation rank | `ReputationRank` | Newcomer, Neighbor, Trusted, Pillar, Hero or Legend. |
| Player level | `PlayerLevel` | Level from 1 to 20 earned from total skill XP. |
| Skill experience | `SkillExperience` | Experience per life skill. |
| Price table | `PriceTable` | Base sell prices and shop buy multipliers. |
| Material id | `MaterialId` | The id of an inventory material. |
| Shop | `Shop` | A vendor with stock, schedule and restock. |
| Bundle | `Bundle` | A Town Hall item set that gives a reward when filled. |
| Museum | `Museum` | The collection of donated items. |
| Milestone | `Milestone` | A one-time reward. |
| Mailbox | `Mailbox` | The homestead box that receives letters and rewards. |

## UI

| Term | Code form | Meaning |
| --- | --- | --- |
| Inventory | `Inventory` | The player's slots of carried items. |
| Inventory slot | `InventorySlot` | One slot holding a material, quality and count. |
| Input map | `InputMap` | Action-based bindings for keyboard, mouse and gamepad. |
| Action | `Action` | One rebindable input action. |
| Interface theme | `InterfaceTheme` | Shared frames, fonts and colors. |
| Item icon | `ItemIcon` | The icon with quality star and count. |
| Tooltip | `Tooltip` | Item details shown on hover. |
| Material source list | `MaterialSourceList` | Where a material can be obtained. |
| Stat change | `StatChange` | Green or red stat change arrows. |
| Toast | `Toast` | A queued short notification. |
| Confirmation dialog | `ConfirmationDialog` | A yes or no prompt. |
| Heads-up display | `HeadsUpDisplay` | The on-screen status display. |
| Monster Field Guide | `FieldGuide` | Pages of monster knowledge revealed by play. |
| Gear Stash | `GearStash` | Storage for gear pieces. |
| Gear compare | `GearCompare` | Side-by-side current and candidate gear. |
| Loadout | `Loadout` | A saved gear set. |
| Assist | `assists` | An accessibility option such as auto-guard or target lock. |

## Audio

| Term | Code form | Meaning |
| --- | --- | --- |
| Audio bus | `AudioBus` | A volume group such as music or effects. |
| Music director | `MusicDirector` | Chooses tracks, layers, crossfades and stingers. |
| Ambience bed | `AmbienceBed` | Layered background sound per zone, phase and weather. |
| Sound effect registry | `SoundEffectRegistry` | Map of cue ids to files and playback rules. |
| Cue | `cue` | A named sound with its variants. |
| Animation sound effect binding | `AnimationSoundEffectBinding` | Plays a cue on a given sprite set frame. |
| Spatial sound effect | `SpatialSoundEffect` | Pan and fade by distance from the player. |
| Captions | `Captions` | On-screen text for sounds. |

## Art

| Term | Code form | Meaning |
| --- | --- | --- |
| Sprite set | `sprite_set` | One animation of frames for one action. |
| Sprite registry | `sprites.ron` | The single registry of sprite sets. |
| Animation frame | `AnimationFrame`, `animation_frame` | One image in a sprite set. |
| Canvas | `canvas` | The pixel area an action frame is drawn on. |
| Facing | `facing` | Direction drawn; right-authored, left by flip. |
| Foot pin | `FOOT_PIN_X` | The anchor that keeps the hunter body in place. |
| Tile | `TILE` | The 32-pixel world grid unit. |
| Portrait | `portrait` | A villager face image with expressions. |
| Season skin | `season_skin` | A tileset variant baked for one season. |
| Palette | `palette` | The shared master colors. |
| Visual effect | `visual_effect` | A drawn effect such as a hit spark or puff. |

## Saves

| Term | Code form | Meaning |
| --- | --- | --- |
| Save file | `SaveFile` | The header plus the player profile. |
| Save header | `SaveHeader` | Name, playtime, calendar, location and gold stored with a profile. |
| Save calendar | `SaveCalendar` | The calendar day and day phase stored on a save header. |
| Save location | `SaveLocation` | Homestead, Forest, Mine or Dungeon stored on a save header. |
| Homestead save | `HomesteadSave` | Tool energy and crop plots stored on a player profile. |
| Mine save | `MineSave` | Depleted ore nodes stored on a player profile. |
| Player profile | `PlayerProfile` | All saved player state in namespaced sub-structs. |
| Profile | `profile` | One save slot picked on the title screen. |
| Migration | `Migrations` | Ordered transforms from one save version to the next. |
| Save version | `version` | The schema number of a save. |
| Autosave | `autosave` | A save written automatically. |
| Backup | `backup` | One of the three most recent rotated copies of a profile. |

## Words not to use

| Do not use | Use instead |
| --- | --- |
| Iridium | Destiny, the top crop quality |
| `sns`, SnS | `sword_and_shield`, `SwordAndShield` |
| `gs`, GS | `great_sword`, `GreatSword` |
| `ls`, LS | `long_sword`, `LongSword` |
| `db`, DB | `dual_blades`, `DualBlades` |
| Sword, as a weapon family | `SwordAndShield` |
| Knight, as the side-scroll character | `Hunter` |
| Avatar, character | `Dwarf` for the top-down body, `Hunter` for the side-scroll body, `Player` for the person |
| `hr`, HR | `hunter_rank` |
| `rep`, Rep | `reputation` |
| `deco` | `decoration` |
| `lv`, `lvl` | `level` |
| `fert_tier` | `fertilizer_tier` |
| `dmg` | `damage` |
| `atk` | `attack` |
| `para` | `paralysis` |
| `xp`, XP | `SkillExperience` or `PlayerLevel` |
| GDD | the design doc |
| `Seeds`, as the sowing tool | `SeedBag` |
| `WorldProgress` | `StoryFlags` or `world_flags` |
| `skill_bindings` | `input_bindings` |
| `WeaponKind`, `ArmorKind` | `GearPiece` with a `GearId` |
| `B20`, `F20` style floor names | `mine_floor` with a number |
| `floor`, Floor, on its own; `F1`; `B1` | `MineFloor`, `mine_floor` plus a number |
| Stamina, for tool use | `tool_energy` |
| Energy, for combat | `Stamina` |
| `frame`, Frame, on its own | `animation_frame` for one image in a sprite set, `frame_material` for the forging material role |
| `def` | `definition` |
| `sfx` | `sound_effect`, `SoundEffect` |
| `vfx` | `visual_effect`, `VisualEffect` |
| `ui`, UI | `interface`, `Interface` |
| `hud`, HUD | `heads_up_display`, `HeadsUpDisplay` |
| `delta` | `change` |
| `confirm`, as a name | `confirmation` |
