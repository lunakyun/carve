//! Vanilla block and item tags, flattened to names. Generated from Pumpkin's tag
//! registry at the pinned commit; do not edit by hand. Sorted by tag id.

pub static BLOCK_TAGS: &[(&str, &str)] = &[
    ("c:barrels", "barrel"),
    ("c:barrels/wooden", "barrel"),
    (
        "c:bars",
        "copper_bars exposed_copper_bars iron_bars oxidized_copper_bars waxed_copper_bars waxed_exposed_copper_bars waxed_oxidized_copper_bars waxed_weathered_copper_bars weathered_copper_bars",
    ),
    (
        "c:bars/copper",
        "copper_bars exposed_copper_bars oxidized_copper_bars waxed_copper_bars waxed_exposed_copper_bars waxed_oxidized_copper_bars waxed_weathered_copper_bars weathered_copper_bars",
    ),
    ("c:bars/iron", "iron_bars"),
    ("c:bookshelves", "bookshelf"),
    ("c:budding_blocks", "budding_amethyst"),
    (
        "c:buds",
        "large_amethyst_bud medium_amethyst_bud small_amethyst_bud",
    ),
    (
        "c:chains",
        "copper_chain exposed_copper_chain iron_chain oxidized_copper_chain waxed_copper_chain waxed_exposed_copper_chain waxed_oxidized_copper_chain waxed_weathered_copper_chain weathered_copper_chain",
    ),
    (
        "c:chests",
        "chest copper_chest ender_chest exposed_copper_chest oxidized_copper_chest trapped_chest waxed_copper_chest waxed_exposed_copper_chest waxed_oxidized_copper_chest waxed_weathered_copper_chest weathered_copper_chest",
    ),
    ("c:chests/ender", "ender_chest"),
    ("c:chests/trapped", "trapped_chest"),
    ("c:chests/wooden", "chest trapped_chest"),
    ("c:clusters", "amethyst_cluster"),
    (
        "c:cobblestones",
        "cobbled_deepslate cobblestone infested_cobblestone mossy_cobblestone",
    ),
    ("c:cobblestones/deepslate", "cobbled_deepslate"),
    ("c:cobblestones/infested", "infested_cobblestone"),
    ("c:cobblestones/mossy", "mossy_cobblestone"),
    ("c:cobblestones/normal", "cobblestone"),
    (
        "c:concretes",
        "black_concrete blue_concrete brown_concrete cyan_concrete gray_concrete green_concrete light_blue_concrete light_gray_concrete lime_concrete magenta_concrete orange_concrete pink_concrete purple_concrete red_concrete white_concrete yellow_concrete",
    ),
    (
        "c:dyeable",
        "black_bed black_carpet black_shulker_box black_wool black_wool_slab black_wool_stairs blue_bed blue_carpet blue_shulker_box blue_wool blue_wool_slab blue_wool_stairs brown_bed brown_carpet brown_shulker_box brown_wool brown_wool_slab brown_wool_stairs candle cyan_bed cyan_carpet cyan_shulker_box cyan_wool cyan_wool_slab cyan_wool_stairs glass glass_pane gray_bed gray_carpet gray_shulker_box gray_wool gray_wool_slab gray_wool_stairs green_bed green_carpet green_shulker_box green_wool green_wool_slab green_wool_stairs light_blue_bed light_blue_carpet light_blue_shulker_box light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_bed light_gray_carpet light_gray_shulker_box light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_bed lime_carpet lime_shulker_box lime_wool lime_wool_slab lime_wool_stairs magenta_bed magenta_carpet magenta_shulker_box magenta_wool magenta_wool_slab magenta_wool_stairs orange_bed orange_carpet orange_shulker_box orange_wool orange_wool_slab orange_wool_stairs pink_bed pink_carpet pink_shulker_box pink_wool pink_wool_slab pink_wool_stairs purple_bed purple_carpet purple_shulker_box purple_wool purple_wool_slab purple_wool_stairs red_bed red_carpet red_shulker_box red_wool red_wool_slab red_wool_stairs shulker_box terracotta white_bed white_carpet white_shulker_box white_wool white_wool_slab white_wool_stairs yellow_bed yellow_carpet yellow_shulker_box yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    ("c:dyeable/dynamic", ""),
    (
        "c:dyeable/simple",
        "black_bed black_carpet black_shulker_box black_wool black_wool_slab black_wool_stairs blue_bed blue_carpet blue_shulker_box blue_wool blue_wool_slab blue_wool_stairs brown_bed brown_carpet brown_shulker_box brown_wool brown_wool_slab brown_wool_stairs candle cyan_bed cyan_carpet cyan_shulker_box cyan_wool cyan_wool_slab cyan_wool_stairs glass glass_pane gray_bed gray_carpet gray_shulker_box gray_wool gray_wool_slab gray_wool_stairs green_bed green_carpet green_shulker_box green_wool green_wool_slab green_wool_stairs light_blue_bed light_blue_carpet light_blue_shulker_box light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_bed light_gray_carpet light_gray_shulker_box light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_bed lime_carpet lime_shulker_box lime_wool lime_wool_slab lime_wool_stairs magenta_bed magenta_carpet magenta_shulker_box magenta_wool magenta_wool_slab magenta_wool_stairs orange_bed orange_carpet orange_shulker_box orange_wool orange_wool_slab orange_wool_stairs pink_bed pink_carpet pink_shulker_box pink_wool pink_wool_slab pink_wool_stairs purple_bed purple_carpet purple_shulker_box purple_wool purple_wool_slab purple_wool_stairs red_bed red_carpet red_shulker_box red_wool red_wool_slab red_wool_stairs shulker_box terracotta white_bed white_carpet white_shulker_box white_wool white_wool_slab white_wool_stairs yellow_bed yellow_carpet yellow_shulker_box yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "c:dyeable/simple/redyeable",
        "black_bed black_carpet black_shulker_box black_wool black_wool_slab black_wool_stairs blue_bed blue_carpet blue_shulker_box blue_wool blue_wool_slab blue_wool_stairs brown_bed brown_carpet brown_shulker_box brown_wool brown_wool_slab brown_wool_stairs cyan_bed cyan_carpet cyan_shulker_box cyan_wool cyan_wool_slab cyan_wool_stairs gray_bed gray_carpet gray_shulker_box gray_wool gray_wool_slab gray_wool_stairs green_bed green_carpet green_shulker_box green_wool green_wool_slab green_wool_stairs light_blue_bed light_blue_carpet light_blue_shulker_box light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_bed light_gray_carpet light_gray_shulker_box light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_bed lime_carpet lime_shulker_box lime_wool lime_wool_slab lime_wool_stairs magenta_bed magenta_carpet magenta_shulker_box magenta_wool magenta_wool_slab magenta_wool_stairs orange_bed orange_carpet orange_shulker_box orange_wool orange_wool_slab orange_wool_stairs pink_bed pink_carpet pink_shulker_box pink_wool pink_wool_slab pink_wool_stairs purple_bed purple_carpet purple_shulker_box purple_wool purple_wool_slab purple_wool_stairs red_bed red_carpet red_shulker_box red_wool red_wool_slab red_wool_stairs white_bed white_carpet white_shulker_box white_wool white_wool_slab white_wool_stairs yellow_bed yellow_carpet yellow_shulker_box yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "c:dyeable/simple/undyed",
        "candle glass glass_pane shulker_box terracotta",
    ),
    (
        "c:dyed",
        "black_banner black_bed black_candle black_carpet black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blue_banner blue_bed blue_candle blue_carpet blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs brown_banner brown_bed brown_candle brown_carpet brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs cyan_banner cyan_bed cyan_candle cyan_carpet cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs gray_banner gray_bed gray_candle gray_carpet gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle green_carpet green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs light_blue_banner light_blue_bed light_blue_candle light_blue_carpet light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle light_gray_carpet light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_banner lime_bed lime_candle lime_carpet lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs magenta_banner magenta_bed magenta_candle magenta_carpet magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs orange_banner orange_bed orange_candle orange_carpet orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs pink_banner pink_bed pink_candle pink_carpet pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs purple_banner purple_bed purple_candle purple_carpet purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs red_banner red_bed red_candle red_carpet red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs white_banner white_bed white_candle white_carpet white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle yellow_carpet yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "c:dyed/black",
        "black_banner black_bed black_candle black_carpet black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs",
    ),
    (
        "c:dyed/blue",
        "blue_banner blue_bed blue_candle blue_carpet blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs",
    ),
    (
        "c:dyed/brown",
        "brown_banner brown_bed brown_candle brown_carpet brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs",
    ),
    (
        "c:dyed/cyan",
        "cyan_banner cyan_bed cyan_candle cyan_carpet cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs",
    ),
    (
        "c:dyed/gray",
        "gray_banner gray_bed gray_candle gray_carpet gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs",
    ),
    (
        "c:dyed/green",
        "green_banner green_bed green_candle green_carpet green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs",
    ),
    (
        "c:dyed/light_blue",
        "light_blue_banner light_blue_bed light_blue_candle light_blue_carpet light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs",
    ),
    (
        "c:dyed/light_gray",
        "light_gray_banner light_gray_bed light_gray_candle light_gray_carpet light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs",
    ),
    (
        "c:dyed/lime",
        "lime_banner lime_bed lime_candle lime_carpet lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs",
    ),
    (
        "c:dyed/magenta",
        "magenta_banner magenta_bed magenta_candle magenta_carpet magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs",
    ),
    (
        "c:dyed/orange",
        "orange_banner orange_bed orange_candle orange_carpet orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs",
    ),
    (
        "c:dyed/pink",
        "pink_banner pink_bed pink_candle pink_carpet pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs",
    ),
    (
        "c:dyed/purple",
        "purple_banner purple_bed purple_candle purple_carpet purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs",
    ),
    (
        "c:dyed/red",
        "red_banner red_bed red_candle red_carpet red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs",
    ),
    (
        "c:dyed/white",
        "white_banner white_bed white_candle white_carpet white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs",
    ),
    (
        "c:dyed/yellow",
        "yellow_banner yellow_bed yellow_candle yellow_carpet yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    ("c:end_stones", "end_stone"),
    (
        "c:fence_gates",
        "acacia_fence_gate bamboo_fence_gate birch_fence_gate cherry_fence_gate crimson_fence_gate dark_oak_fence_gate jungle_fence_gate mangrove_fence_gate oak_fence_gate pale_oak_fence_gate spruce_fence_gate warped_fence_gate",
    ),
    (
        "c:fence_gates/wooden",
        "acacia_fence_gate bamboo_fence_gate birch_fence_gate cherry_fence_gate crimson_fence_gate dark_oak_fence_gate jungle_fence_gate mangrove_fence_gate oak_fence_gate pale_oak_fence_gate spruce_fence_gate warped_fence_gate",
    ),
    (
        "c:fences",
        "acacia_fence bamboo_fence birch_fence cherry_fence crimson_fence dark_oak_fence jungle_fence mangrove_fence nether_brick_fence oak_fence pale_oak_fence spruce_fence warped_fence",
    ),
    ("c:fences/nether_brick", "nether_brick_fence"),
    (
        "c:fences/wooden",
        "acacia_fence bamboo_fence birch_fence cherry_fence crimson_fence dark_oak_fence jungle_fence mangrove_fence oak_fence pale_oak_fence spruce_fence warped_fence",
    ),
    (
        "c:flowers",
        "allium azure_bluet blue_orchid cactus_flower cherry_leaves chorus_flower closed_eyeblossom cornflower dandelion flowering_azalea flowering_azalea_leaves lilac lily_of_the_valley mangrove_propagule open_eyeblossom orange_tulip oxeye_daisy peony pink_petals pink_tulip pitcher_plant poppy red_tulip rose_bush spore_blossom sunflower torchflower white_tulip wildflowers wither_rose",
    ),
    (
        "c:flowers/small",
        "allium azure_bluet blue_orchid closed_eyeblossom cornflower dandelion lily_of_the_valley open_eyeblossom orange_tulip oxeye_daisy pink_tulip poppy red_tulip torchflower white_tulip wither_rose",
    ),
    ("c:flowers/tall", "lilac peony pitcher_plant rose_bush sunflower"),
    (
        "c:froglights",
        "ochre_froglight pearlescent_froglight verdant_froglight",
    ),
    (
        "c:glass_blocks",
        "black_stained_glass blue_stained_glass brown_stained_glass cyan_stained_glass glass gray_stained_glass green_stained_glass light_blue_stained_glass light_gray_stained_glass lime_stained_glass magenta_stained_glass orange_stained_glass pink_stained_glass purple_stained_glass red_stained_glass tinted_glass white_stained_glass yellow_stained_glass",
    ),
    (
        "c:glass_blocks/cheap",
        "black_stained_glass blue_stained_glass brown_stained_glass cyan_stained_glass glass gray_stained_glass green_stained_glass light_blue_stained_glass light_gray_stained_glass lime_stained_glass magenta_stained_glass orange_stained_glass pink_stained_glass purple_stained_glass red_stained_glass white_stained_glass yellow_stained_glass",
    ),
    ("c:glass_blocks/colorless", "glass"),
    ("c:glass_blocks/tinted", "tinted_glass"),
    (
        "c:glass_panes",
        "black_stained_glass_pane blue_stained_glass_pane brown_stained_glass_pane cyan_stained_glass_pane glass_pane gray_stained_glass_pane green_stained_glass_pane light_blue_stained_glass_pane light_gray_stained_glass_pane lime_stained_glass_pane magenta_stained_glass_pane orange_stained_glass_pane pink_stained_glass_pane purple_stained_glass_pane red_stained_glass_pane white_stained_glass_pane yellow_stained_glass_pane",
    ),
    ("c:glass_panes/colorless", "glass_pane"),
    (
        "c:glazed_terracottas",
        "black_glazed_terracotta blue_glazed_terracotta brown_glazed_terracotta cyan_glazed_terracotta gray_glazed_terracotta green_glazed_terracotta light_blue_glazed_terracotta light_gray_glazed_terracotta lime_glazed_terracotta magenta_glazed_terracotta orange_glazed_terracotta pink_glazed_terracotta purple_glazed_terracotta red_glazed_terracotta white_glazed_terracotta yellow_glazed_terracotta",
    ),
    ("c:gravels", "gravel"),
    ("c:hidden_from_recipe_viewers", ""),
    (
        "c:natural_logs",
        "acacia_log bamboo_block birch_log cherry_log crimson_stem dark_oak_log jungle_log mangrove_log oak_log pale_oak_log poplar_log spruce_log warped_stem",
    ),
    ("c:natural_logs/nether", "crimson_stem warped_stem"),
    (
        "c:natural_logs/overworld",
        "acacia_log bamboo_block birch_log cherry_log dark_oak_log jungle_log mangrove_log oak_log pale_oak_log poplar_log spruce_log",
    ),
    (
        "c:natural_woods",
        "acacia_wood birch_wood cherry_wood crimson_hyphae dark_oak_wood jungle_wood mangrove_wood oak_wood pale_oak_wood poplar_wood spruce_wood warped_hyphae",
    ),
    ("c:netherracks", "netherrack"),
    ("c:obsidians", "crying_obsidian obsidian"),
    ("c:obsidians/crying", "crying_obsidian"),
    ("c:obsidians/normal", "obsidian"),
    ("c:ore_bearing_ground/deepslate", "deepslate"),
    ("c:ore_bearing_ground/netherrack", "netherrack"),
    ("c:ore_bearing_ground/stone", "stone"),
    (
        "c:ore_rates/dense",
        "copper_ore deepslate_copper_ore deepslate_lapis_ore deepslate_redstone_ore lapis_ore redstone_ore",
    ),
    (
        "c:ore_rates/singular",
        "ancient_debris coal_ore deepslate_coal_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore diamond_ore emerald_ore gold_ore iron_ore nether_quartz_ore",
    ),
    ("c:ore_rates/sparse", "nether_gold_ore"),
    (
        "c:ores",
        "ancient_debris coal_ore copper_ore deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore diamond_ore emerald_ore gold_ore iron_ore lapis_ore nether_gold_ore nether_quartz_ore redstone_ore",
    ),
    ("c:ores/coal", "coal_ore deepslate_coal_ore"),
    ("c:ores/copper", "copper_ore deepslate_copper_ore"),
    ("c:ores/diamond", "deepslate_diamond_ore diamond_ore"),
    ("c:ores/emerald", "deepslate_emerald_ore emerald_ore"),
    ("c:ores/gold", "deepslate_gold_ore gold_ore nether_gold_ore"),
    ("c:ores/iron", "deepslate_iron_ore iron_ore"),
    ("c:ores/lapis", "deepslate_lapis_ore lapis_ore"),
    ("c:ores/netherite_scrap", "ancient_debris"),
    ("c:ores/quartz", "nether_quartz_ore"),
    ("c:ores/redstone", "deepslate_redstone_ore redstone_ore"),
    (
        "c:ores_in_ground/deepslate",
        "deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore",
    ),
    ("c:ores_in_ground/netherrack", "nether_gold_ore nether_quartz_ore"),
    (
        "c:ores_in_ground/stone",
        "coal_ore copper_ore diamond_ore emerald_ore gold_ore iron_ore lapis_ore redstone_ore",
    ),
    ("c:player_workstations/crafting_tables", "crafting_table"),
    ("c:player_workstations/furnaces", "furnace"),
    ("c:pumpkins", "carved_pumpkin jack_o_lantern pumpkin"),
    ("c:pumpkins/carved", "carved_pumpkin"),
    ("c:pumpkins/jack_o_lanterns", "jack_o_lantern"),
    ("c:pumpkins/normal", "pumpkin"),
    ("c:relocation_not_supported", ""),
    ("c:ropes", ""),
    ("c:sands", "red_sand sand"),
    ("c:sands/colorless", "sand"),
    ("c:sands/red", "red_sand"),
    (
        "c:sandstone/blocks",
        "chiseled_red_sandstone chiseled_sandstone cut_red_sandstone cut_sandstone red_sandstone sandstone smooth_red_sandstone smooth_sandstone",
    ),
    (
        "c:sandstone/red_blocks",
        "chiseled_red_sandstone cut_red_sandstone red_sandstone smooth_red_sandstone",
    ),
    (
        "c:sandstone/red_slabs",
        "cut_red_sandstone_slab red_sandstone_slab smooth_red_sandstone_slab",
    ),
    (
        "c:sandstone/red_stairs",
        "red_sandstone_stairs smooth_red_sandstone_stairs",
    ),
    (
        "c:sandstone/slabs",
        "cut_red_sandstone_slab cut_sandstone_slab red_sandstone_slab sandstone_slab smooth_red_sandstone_slab smooth_sandstone_slab",
    ),
    (
        "c:sandstone/stairs",
        "red_sandstone_stairs sandstone_stairs smooth_red_sandstone_stairs smooth_sandstone_stairs",
    ),
    (
        "c:sandstone/uncolored_blocks",
        "chiseled_sandstone cut_sandstone sandstone smooth_sandstone",
    ),
    (
        "c:sandstone/uncolored_slabs",
        "cut_sandstone_slab sandstone_slab smooth_sandstone_slab",
    ),
    (
        "c:sandstone/uncolored_stairs",
        "sandstone_stairs smooth_sandstone_stairs",
    ),
    (
        "c:skulls",
        "creeper_head creeper_wall_head dragon_head dragon_wall_head piglin_head piglin_wall_head player_head player_wall_head skeleton_skull skeleton_wall_skull wither_skeleton_skull wither_skeleton_wall_skull zombie_head zombie_wall_head",
    ),
    ("c:stones", "andesite deepslate diorite granite stone tuff"),
    (
        "c:storage_blocks",
        "bone_block coal_block copper_block diamond_block dried_kelp_block emerald_block gold_block hay_block iron_block lapis_block netherite_block raw_copper_block raw_gold_block raw_iron_block redstone_block resin_block slime_block",
    ),
    ("c:storage_blocks/bone_meal", "bone_block"),
    ("c:storage_blocks/coal", "coal_block"),
    ("c:storage_blocks/copper", "copper_block"),
    ("c:storage_blocks/diamond", "diamond_block"),
    ("c:storage_blocks/dried_kelp", "dried_kelp_block"),
    ("c:storage_blocks/emerald", "emerald_block"),
    ("c:storage_blocks/gold", "gold_block"),
    ("c:storage_blocks/iron", "iron_block"),
    ("c:storage_blocks/lapis", "lapis_block"),
    ("c:storage_blocks/netherite", "netherite_block"),
    ("c:storage_blocks/raw_copper", "raw_copper_block"),
    ("c:storage_blocks/raw_gold", "raw_gold_block"),
    ("c:storage_blocks/raw_iron", "raw_iron_block"),
    ("c:storage_blocks/redstone", "redstone_block"),
    ("c:storage_blocks/resin", "resin_block"),
    ("c:storage_blocks/slime", "slime_block"),
    ("c:storage_blocks/wheat", "hay_block"),
    (
        "c:stripped_logs",
        "stripped_acacia_log stripped_bamboo_block stripped_birch_log stripped_cherry_log stripped_crimson_stem stripped_dark_oak_log stripped_jungle_log stripped_mangrove_log stripped_oak_log stripped_pale_oak_log stripped_poplar_log stripped_spruce_log stripped_warped_stem",
    ),
    (
        "c:stripped_woods",
        "stripped_acacia_wood stripped_birch_wood stripped_cherry_wood stripped_crimson_hyphae stripped_dark_oak_wood stripped_jungle_wood stripped_mangrove_wood stripped_oak_wood stripped_pale_oak_wood stripped_poplar_wood stripped_spruce_wood stripped_warped_hyphae",
    ),
    (
        "c:villager_job_sites",
        "barrel blast_furnace brewing_stand cartography_table cauldron composter fletching_table grindstone lava_cauldron lectern loom powder_snow_cauldron smithing_table smoker stonecutter water_cauldron",
    ),
    (
        "minecraft:acacia_logs",
        "acacia_log acacia_wood stripped_acacia_log stripped_acacia_wood",
    ),
    ("minecraft:air", "air cave_air void_air"),
    (
        "minecraft:all_hanging_signs",
        "acacia_hanging_sign acacia_wall_hanging_sign bamboo_hanging_sign bamboo_wall_hanging_sign birch_hanging_sign birch_wall_hanging_sign cherry_hanging_sign cherry_wall_hanging_sign crimson_hanging_sign crimson_wall_hanging_sign dark_oak_hanging_sign dark_oak_wall_hanging_sign jungle_hanging_sign jungle_wall_hanging_sign mangrove_hanging_sign mangrove_wall_hanging_sign oak_hanging_sign oak_wall_hanging_sign pale_oak_hanging_sign pale_oak_wall_hanging_sign poplar_hanging_sign poplar_wall_hanging_sign spruce_hanging_sign spruce_wall_hanging_sign warped_hanging_sign warped_wall_hanging_sign",
    ),
    (
        "minecraft:all_signs",
        "acacia_hanging_sign acacia_sign acacia_wall_hanging_sign acacia_wall_sign bamboo_hanging_sign bamboo_sign bamboo_wall_hanging_sign bamboo_wall_sign birch_hanging_sign birch_sign birch_wall_hanging_sign birch_wall_sign cherry_hanging_sign cherry_sign cherry_wall_hanging_sign cherry_wall_sign crimson_hanging_sign crimson_sign crimson_wall_hanging_sign crimson_wall_sign dark_oak_hanging_sign dark_oak_sign dark_oak_wall_hanging_sign dark_oak_wall_sign jungle_hanging_sign jungle_sign jungle_wall_hanging_sign jungle_wall_sign mangrove_hanging_sign mangrove_sign mangrove_wall_hanging_sign mangrove_wall_sign oak_hanging_sign oak_sign oak_wall_hanging_sign oak_wall_sign pale_oak_hanging_sign pale_oak_sign pale_oak_wall_hanging_sign pale_oak_wall_sign poplar_hanging_sign poplar_sign poplar_wall_hanging_sign poplar_wall_sign spruce_hanging_sign spruce_sign spruce_wall_hanging_sign spruce_wall_sign warped_hanging_sign warped_sign warped_wall_hanging_sign warped_wall_sign",
    ),
    (
        "minecraft:ancient_city_replaceable",
        "cobbled_deepslate cracked_deepslate_bricks cracked_deepslate_tiles deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_tile_slab deepslate_tile_wall deepslate_tiles gray_wool",
    ),
    ("minecraft:animals_spawnable_on", "grass_block"),
    ("minecraft:anvil", "anvil chipped_anvil damaged_anvil"),
    (
        "minecraft:armadillo_spawnable_on",
        "brown_terracotta coarse_dirt grass_block light_gray_terracotta orange_terracotta red_sand red_terracotta terracotta white_terracotta yellow_terracotta",
    ),
    ("minecraft:axolotls_spawnable_on", "clay"),
    (
        "minecraft:azalea_grows_on",
        "black_terracotta blue_terracotta brown_terracotta coarse_dirt cyan_terracotta dirt grass_block gray_terracotta green_terracotta light_blue_terracotta light_gray_terracotta lime_terracotta magenta_terracotta moss_block mud muddy_mangrove_roots mycelium orange_terracotta pale_moss_block pink_terracotta podzol powder_snow purple_terracotta red_sand red_terracotta rooted_dirt sand snow_block suspicious_sand terracotta white_terracotta yellow_terracotta",
    ),
    (
        "minecraft:azalea_root_replaceable",
        "andesite black_terracotta blue_terracotta brown_terracotta clay coarse_dirt cyan_terracotta deepslate diorite dirt granite grass_block gravel gray_terracotta green_terracotta light_blue_terracotta light_gray_terracotta lime_terracotta magenta_terracotta moss_block mud muddy_mangrove_roots mycelium orange_terracotta pale_moss_block pink_terracotta podzol powder_snow purple_terracotta red_sand red_terracotta rooted_dirt sand snow_block stone terracotta tuff white_terracotta yellow_terracotta",
    ),
    (
        "minecraft:badlands_terracotta",
        "brown_terracotta light_gray_terracotta orange_terracotta red_terracotta terracotta white_terracotta yellow_terracotta",
    ),
    ("minecraft:bamboo_blocks", "bamboo_block stripped_bamboo_block"),
    (
        "minecraft:banners",
        "black_banner black_wall_banner blue_banner blue_wall_banner brown_banner brown_wall_banner cyan_banner cyan_wall_banner gray_banner gray_wall_banner green_banner green_wall_banner light_blue_banner light_blue_wall_banner light_gray_banner light_gray_wall_banner lime_banner lime_wall_banner magenta_banner magenta_wall_banner orange_banner orange_wall_banner pink_banner pink_wall_banner purple_banner purple_wall_banner red_banner red_wall_banner white_banner white_wall_banner yellow_banner yellow_wall_banner",
    ),
    (
        "minecraft:bars",
        "copper_bars exposed_copper_bars iron_bars oxidized_copper_bars waxed_copper_bars waxed_exposed_copper_bars waxed_oxidized_copper_bars waxed_weathered_copper_bars weathered_copper_bars",
    ),
    ("minecraft:base_stone_nether", "basalt blackstone netherrack"),
    (
        "minecraft:base_stone_overworld",
        "andesite deepslate diorite granite stone tuff",
    ),
    (
        "minecraft:bats_spawnable_on",
        "andesite deepslate diorite granite stone tuff",
    ),
    (
        "minecraft:beacon_base_blocks",
        "diamond_block emerald_block gold_block iron_block netherite_block",
    ),
    (
        "minecraft:beds",
        "black_bed blue_bed brown_bed cyan_bed gray_bed green_bed light_blue_bed light_gray_bed lime_bed magenta_bed orange_bed pink_bed purple_bed red_bed white_bed yellow_bed",
    ),
    (
        "minecraft:bee_attractive",
        "allium azure_bluet blue_orchid cactus_flower cherry_leaves chorus_flower cornflower dandelion flowering_azalea flowering_azalea_leaves lilac lily_of_the_valley mangrove_propagule open_eyeblossom orange_tulip oxeye_daisy peony pink_petals pink_tulip pitcher_plant poppy red_tulip rose_bush spore_blossom sunflower torchflower white_tulip wildflowers wither_rose",
    ),
    (
        "minecraft:bee_growables",
        "beetroots carrots cave_vines cave_vines_plant melon_stem pitcher_crop potatoes pumpkin_stem sweet_berry_bush torchflower_crop wheat",
    ),
    ("minecraft:beehives", "bee_nest beehive"),
    (
        "minecraft:beneath_bamboo_podzol_replaceable",
        "coarse_dirt dirt grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt",
    ),
    (
        "minecraft:beneath_tree_podzol_replaceable",
        "coarse_dirt dirt grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt",
    ),
    (
        "minecraft:birch_logs",
        "birch_log birch_wood stripped_birch_log stripped_birch_wood",
    ),
    (
        "minecraft:blocks_dolphin_jump",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_leaves acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil azalea_leaves bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_leaves birch_log birch_planks birch_pressure_plate birch_shelf birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_leaves cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_leaves dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table flowering_azalea_leaves frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_leaves jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_leaves mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_leaves oak_log oak_planks oak_pressure_plate oak_shelf oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_poplar_leaves orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_leaves pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_poplar_leaves red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_leaves spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_poplar_leaves yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:blocks_fluid_flow",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_leaves acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_sign acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wall_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil azalea_leaves bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_sign bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign bamboo_wall_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_leaves birch_log birch_planks birch_pressure_plate birch_shelf birch_sign birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wall_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_leaves cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_sign cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wall_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_sign crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crimson_wall_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_leaves dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_sign dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wall_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table flowering_azalea_leaves frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_leaves jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_sign jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wall_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_leaves mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_sign mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wall_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_leaves oak_log oak_planks oak_pressure_plate oak_shelf oak_sign oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wall_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_poplar_leaves orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_leaves pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_sign pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wall_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_sign poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wall_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_poplar_leaves red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_leaves spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_sign spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wall_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_sign warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wall_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_poplar_leaves yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:blocks_lava_fire_spread",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_leaves acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil azalea_leaves bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_leaves birch_log birch_planks birch_pressure_plate birch_shelf birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_leaves cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_leaves dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table flowering_azalea_leaves frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_leaves jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_leaves mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_leaves oak_log oak_planks oak_pressure_plate oak_shelf oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_poplar_leaves orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_leaves pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_poplar_leaves red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_leaves spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_poplar_leaves yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:blocks_motion",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_leaves acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil azalea_leaves bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_leaves birch_log birch_planks birch_pressure_plate birch_shelf birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_leaves cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_leaves dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table flowering_azalea_leaves frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_leaves jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_leaves mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_leaves oak_log oak_planks oak_pressure_plate oak_shelf oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_poplar_leaves orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_leaves pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_poplar_leaves red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_leaves spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_poplar_leaves yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:blocks_motion_in_heightmap",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_leaves acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil azalea_leaves bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_leaves birch_log birch_planks birch_pressure_plate birch_shelf birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_leaves cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_leaves dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table flowering_azalea_leaves frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_leaves jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_leaves mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_leaves oak_log oak_planks oak_pressure_plate oak_shelf oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_poplar_leaves orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_leaves pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_poplar_leaves red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_leaves spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_poplar_leaves yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:blocks_motion_in_heightmap_no_leaves",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_log birch_planks birch_pressure_plate birch_shelf birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_log oak_planks oak_pressure_plate oak_shelf oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:blocks_motion_no_leaves",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_log birch_planks birch_pressure_plate birch_shelf birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_log oak_planks oak_pressure_plate oak_shelf oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    ("minecraft:blocks_wind_charge_explosions", "barrier bedrock"),
    (
        "minecraft:buttons",
        "acacia_button bamboo_button birch_button cherry_button crimson_button dark_oak_button jungle_button mangrove_button oak_button pale_oak_button polished_blackstone_button poplar_button spruce_button stone_button warped_button",
    ),
    (
        "minecraft:camel_sand_step_sound_blocks",
        "black_concrete_powder blue_concrete_powder brown_concrete_powder cyan_concrete_powder gray_concrete_powder green_concrete_powder light_blue_concrete_powder light_gray_concrete_powder lime_concrete_powder magenta_concrete_powder orange_concrete_powder pink_concrete_powder purple_concrete_powder red_concrete_powder red_sand sand suspicious_sand white_concrete_powder yellow_concrete_powder",
    ),
    ("minecraft:camels_spawnable_on", "red_sand sand suspicious_sand"),
    ("minecraft:campfires", "campfire soul_campfire"),
    (
        "minecraft:can_glide_through",
        "cave_vines cave_vines_plant twisting_vines twisting_vines_plant vine weeping_vines weeping_vines_plant",
    ),
    (
        "minecraft:candle_cakes",
        "black_candle_cake blue_candle_cake brown_candle_cake candle_cake cyan_candle_cake gray_candle_cake green_candle_cake light_blue_candle_cake light_gray_candle_cake lime_candle_cake magenta_candle_cake orange_candle_cake pink_candle_cake purple_candle_cake red_candle_cake white_candle_cake yellow_candle_cake",
    ),
    (
        "minecraft:candles",
        "black_candle blue_candle brown_candle candle cyan_candle gray_candle green_candle light_blue_candle light_gray_candle lime_candle magenta_candle orange_candle pink_candle purple_candle red_candle white_candle yellow_candle",
    ),
    (
        "minecraft:cannot_place_basalt_pillar_on",
        "bedrock chest lava magma_block nether_brick_fence nether_brick_stairs nether_bricks nether_wart soul_sand spawner",
    ),
    (
        "minecraft:cannot_replace_below_tree_trunk",
        "coarse_dirt dirt moss_block mud muddy_mangrove_roots pale_moss_block podzol rooted_dirt",
    ),
    ("minecraft:cannot_support_kelp", "magma_block"),
    ("minecraft:cannot_support_seagrass", "magma_block"),
    ("minecraft:cannot_support_snow_layer", "barrier ice packed_ice"),
    (
        "minecraft:cat_does_not_teleport_to",
        "cactus campfire fire lava_cauldron magma_block pointed_dripstone powder_snow soul_campfire soul_fire sweet_berry_bush wither_rose",
    ),
    (
        "minecraft:cats_can_lie_on",
        "black_bed blue_bed brown_bed cyan_bed gray_bed green_bed light_blue_bed light_gray_bed lime_bed magenta_bed orange_bed pink_bed purple_bed red_bed white_bed yellow_bed",
    ),
    (
        "minecraft:cats_can_sit_on",
        "black_bed blue_bed brown_bed chest cyan_bed furnace gray_bed green_bed light_blue_bed light_gray_bed lime_bed magenta_bed orange_bed pink_bed purple_bed red_bed white_bed yellow_bed",
    ),
    (
        "minecraft:cauldrons",
        "cauldron lava_cauldron powder_snow_cauldron water_cauldron",
    ),
    ("minecraft:causes_continuous_geyser_eruptions", "lava"),
    ("minecraft:causes_periodic_geyser_eruptions", "magma_block"),
    (
        "minecraft:causes_suffocation",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_leaves acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil azalea_leaves bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_leaves birch_log birch_planks birch_pressure_plate birch_shelf birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_leaves cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_leaves dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table flowering_azalea_leaves frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_leaves jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_leaves mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_leaves oak_log oak_planks oak_pressure_plate oak_shelf oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_poplar_leaves orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_leaves pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_poplar_leaves red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_leaves spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_poplar_leaves yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    ("minecraft:cave_vines", "cave_vines cave_vines_plant"),
    (
        "minecraft:ceiling_hanging_signs",
        "acacia_hanging_sign bamboo_hanging_sign birch_hanging_sign cherry_hanging_sign crimson_hanging_sign dark_oak_hanging_sign jungle_hanging_sign mangrove_hanging_sign oak_hanging_sign pale_oak_hanging_sign poplar_hanging_sign spruce_hanging_sign warped_hanging_sign",
    ),
    (
        "minecraft:chains",
        "copper_chain exposed_copper_chain iron_chain oxidized_copper_chain waxed_copper_chain waxed_exposed_copper_chain waxed_oxidized_copper_chain waxed_weathered_copper_chain weathered_copper_chain",
    ),
    (
        "minecraft:cherry_logs",
        "cherry_log cherry_wood stripped_cherry_log stripped_cherry_wood",
    ),
    (
        "minecraft:climbable",
        "cave_vines cave_vines_plant ladder scaffolding twisting_vines twisting_vines_plant vine weeping_vines weeping_vines_plant",
    ),
    ("minecraft:coal_ores", "coal_ore deepslate_coal_ore"),
    (
        "minecraft:combination_step_sound_blocks",
        "black_carpet blue_carpet brown_carpet crimson_roots cyan_carpet gray_carpet green_carpet light_blue_carpet light_gray_carpet lime_carpet magenta_carpet moss_carpet nether_sprouts orange_carpet pale_moss_carpet pink_carpet purple_carpet red_carpet resin_clump snow warped_roots white_carpet yellow_carpet",
    ),
    (
        "minecraft:completes_find_tree_tutorial",
        "acacia_leaves acacia_log acacia_wood azalea_leaves birch_leaves birch_log birch_wood cherry_leaves cherry_log cherry_wood crimson_hyphae crimson_stem dark_oak_leaves dark_oak_log dark_oak_wood flowering_azalea_leaves jungle_leaves jungle_log jungle_wood mangrove_leaves mangrove_log mangrove_wood nether_wart_block oak_leaves oak_log oak_wood orange_poplar_leaves pale_oak_leaves pale_oak_log pale_oak_wood poplar_log poplar_wood red_poplar_leaves spruce_leaves spruce_log spruce_wood stripped_acacia_log stripped_acacia_wood stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem warped_hyphae warped_stem warped_wart_block yellow_poplar_leaves",
    ),
    (
        "minecraft:concrete",
        "black_concrete blue_concrete brown_concrete cyan_concrete gray_concrete green_concrete light_blue_concrete light_gray_concrete lime_concrete magenta_concrete orange_concrete pink_concrete purple_concrete red_concrete white_concrete yellow_concrete",
    ),
    (
        "minecraft:concrete_powders",
        "black_concrete_powder blue_concrete_powder brown_concrete_powder cyan_concrete_powder gray_concrete_powder green_concrete_powder light_blue_concrete_powder light_gray_concrete_powder lime_concrete_powder magenta_concrete_powder orange_concrete_powder pink_concrete_powder purple_concrete_powder red_concrete_powder white_concrete_powder yellow_concrete_powder",
    ),
    (
        "minecraft:concrete_slabs",
        "black_concrete_slab blue_concrete_slab brown_concrete_slab cyan_concrete_slab gray_concrete_slab green_concrete_slab light_blue_concrete_slab light_gray_concrete_slab lime_concrete_slab magenta_concrete_slab orange_concrete_slab pink_concrete_slab purple_concrete_slab red_concrete_slab white_concrete_slab yellow_concrete_slab",
    ),
    (
        "minecraft:concrete_stairs",
        "black_concrete_stairs blue_concrete_stairs brown_concrete_stairs cyan_concrete_stairs gray_concrete_stairs green_concrete_stairs light_blue_concrete_stairs light_gray_concrete_stairs lime_concrete_stairs magenta_concrete_stairs orange_concrete_stairs pink_concrete_stairs purple_concrete_stairs red_concrete_stairs white_concrete_stairs yellow_concrete_stairs",
    ),
    (
        "minecraft:conduit_effect_block",
        "dark_prismarine prismarine prismarine_bricks sea_lantern",
    ),
    ("minecraft:convertible_to_mud", "coarse_dirt dirt rooted_dirt"),
    (
        "minecraft:copper",
        "copper_block exposed_copper oxidized_copper waxed_copper_block waxed_exposed_copper waxed_oxidized_copper waxed_weathered_copper weathered_copper",
    ),
    (
        "minecraft:copper_chests",
        "copper_chest exposed_copper_chest oxidized_copper_chest waxed_copper_chest waxed_exposed_copper_chest waxed_oxidized_copper_chest waxed_weathered_copper_chest weathered_copper_chest",
    ),
    (
        "minecraft:copper_golem_statues",
        "copper_golem_statue exposed_copper_golem_statue oxidized_copper_golem_statue waxed_copper_golem_statue waxed_exposed_copper_golem_statue waxed_oxidized_copper_golem_statue waxed_weathered_copper_golem_statue weathered_copper_golem_statue",
    ),
    ("minecraft:copper_ores", "copper_ore deepslate_copper_ore"),
    (
        "minecraft:coral_blocks",
        "brain_coral_block bubble_coral_block fire_coral_block horn_coral_block tube_coral_block",
    ),
    (
        "minecraft:coral_plants",
        "brain_coral bubble_coral fire_coral horn_coral tube_coral",
    ),
    (
        "minecraft:corals",
        "brain_coral brain_coral_fan bubble_coral bubble_coral_fan fire_coral fire_coral_fan horn_coral horn_coral_fan tube_coral tube_coral_fan",
    ),
    (
        "minecraft:crimson_stems",
        "crimson_hyphae crimson_stem stripped_crimson_hyphae stripped_crimson_stem",
    ),
    (
        "minecraft:crops",
        "beetroots carrots melon_stem pitcher_crop potatoes pumpkin_stem torchflower_crop wheat",
    ),
    (
        "minecraft:crystal_sound_blocks",
        "amethyst_block budding_amethyst",
    ),
    (
        "minecraft:cushion_uses_collision_shape",
        "cauldron composter hopper lava_cauldron powder_snow_cauldron water_cauldron",
    ),
    (
        "minecraft:dampens_vibrations",
        "black_carpet black_wool black_wool_slab black_wool_stairs blue_carpet blue_wool blue_wool_slab blue_wool_stairs brown_carpet brown_wool brown_wool_slab brown_wool_stairs cyan_carpet cyan_wool cyan_wool_slab cyan_wool_stairs gray_carpet gray_wool gray_wool_slab gray_wool_stairs green_carpet green_wool green_wool_slab green_wool_stairs light_blue_carpet light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_carpet light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_carpet lime_wool lime_wool_slab lime_wool_stairs magenta_carpet magenta_wool magenta_wool_slab magenta_wool_stairs orange_carpet orange_wool orange_wool_slab orange_wool_stairs pink_carpet pink_wool pink_wool_slab pink_wool_stairs purple_carpet purple_wool purple_wool_slab purple_wool_stairs red_carpet red_wool red_wool_slab red_wool_stairs white_carpet white_wool white_wool_slab white_wool_stairs yellow_carpet yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:dangerous_for_teleportation",
        "cactus campfire fire lava_cauldron magma_block pointed_dripstone powder_snow soul_campfire soul_fire sweet_berry_bush wither_rose",
    ),
    (
        "minecraft:dark_oak_logs",
        "dark_oak_log dark_oak_wood stripped_dark_oak_log stripped_dark_oak_wood",
    ),
    ("minecraft:deepslate_ore_replaceables", "deepslate"),
    ("minecraft:default_immune_to", ""),
    ("minecraft:diamond_ores", "deepslate_diamond_ore diamond_ore"),
    ("minecraft:dirt", "coarse_dirt dirt rooted_dirt"),
    ("minecraft:does_not_block_hoppers", "bee_nest beehive"),
    (
        "minecraft:doors",
        "acacia_door bamboo_door birch_door cherry_door copper_door crimson_door dark_oak_door exposed_copper_door iron_door jungle_door mangrove_door oak_door oxidized_copper_door pale_oak_door poplar_door spruce_door warped_door waxed_copper_door waxed_exposed_copper_door waxed_oxidized_copper_door waxed_weathered_copper_door weathered_copper_door",
    ),
    (
        "minecraft:dragon_immune",
        "barrier bedrock chain_command_block command_block crying_obsidian end_gateway end_portal end_portal_frame end_stone iron_bars jigsaw moving_piston obsidian reinforced_deepslate repeating_command_block respawn_anchor structure_block test_block test_instance_block",
    ),
    ("minecraft:dragon_transparent", "fire light soul_fire"),
    (
        "minecraft:dripstone_replaceable_blocks",
        "andesite deepslate diorite granite stone tuff",
    ),
    (
        "minecraft:edible_for_sheep",
        "fern short_dry_grass short_grass tall_dry_grass",
    ),
    ("minecraft:emerald_ores", "deepslate_emerald_ore emerald_ore"),
    ("minecraft:enables_bubble_column_drag_down", "magma_block"),
    ("minecraft:enables_bubble_column_push_up", "soul_sand"),
    ("minecraft:enchantment_power_provider", "bookshelf"),
    (
        "minecraft:enchantment_power_transmitter",
        "air bubble_column bush cave_air crimson_roots dead_bush fern fire glow_lichen hanging_roots large_fern lava leaf_litter light nether_sprouts red_shrub resin_clump seagrass short_dry_grass short_grass snow soul_fire structure_void tall_dry_grass tall_grass tall_seagrass vine void_air warped_roots water",
    ),
    (
        "minecraft:enderman_does_not_teleport_to",
        "bedrock cactus campfire fire lava_cauldron magma_block pointed_dripstone powder_snow soul_campfire soul_fire sweet_berry_bush wither_rose",
    ),
    (
        "minecraft:enderman_holdable",
        "allium azure_bluet blue_orchid brown_mushroom cactus cactus_flower carved_pumpkin clay closed_eyeblossom coarse_dirt cornflower crimson_fungus crimson_nylium crimson_roots dandelion dirt golden_dandelion grass_block gravel lily_of_the_valley melon moss_block mud muddy_mangrove_roots mycelium open_eyeblossom orange_tulip oxeye_daisy pale_moss_block pink_tulip podzol poppy pumpkin red_mushroom red_sand red_tulip rooted_dirt sand tnt torchflower warped_fungus warped_nylium warped_roots white_tulip wither_rose",
    ),
    (
        "minecraft:entities_can_teleport_to",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_leaves acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil azalea_leaves bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_leaves birch_log birch_planks birch_pressure_plate birch_shelf birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_leaves cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_leaves dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table flowering_azalea_leaves frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_leaves jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_leaves mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_leaves oak_log oak_planks oak_pressure_plate oak_shelf oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_poplar_leaves orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_leaves pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_poplar_leaves red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_leaves spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_poplar_leaves yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:fall_damage_resetting",
        "cave_vines cave_vines_plant cobweb ladder scaffolding sweet_berry_bush twisting_vines twisting_vines_plant vine weeping_vines weeping_vines_plant",
    ),
    (
        "minecraft:features_cannot_replace",
        "bedrock chest end_portal_frame reinforced_deepslate spawner trial_spawner vault",
    ),
    (
        "minecraft:fence_gates",
        "acacia_fence_gate bamboo_fence_gate birch_fence_gate cherry_fence_gate crimson_fence_gate dark_oak_fence_gate jungle_fence_gate mangrove_fence_gate oak_fence_gate pale_oak_fence_gate poplar_fence_gate spruce_fence_gate warped_fence_gate",
    ),
    (
        "minecraft:fences",
        "acacia_fence bamboo_fence birch_fence cherry_fence crimson_fence dark_oak_fence jungle_fence mangrove_fence nether_brick_fence oak_fence pale_oak_fence poplar_fence spruce_fence warped_fence",
    ),
    ("minecraft:fire", "fire soul_fire"),
    (
        "minecraft:flower_pots",
        "flower_pot potted_acacia_sapling potted_allium potted_azalea_bush potted_azure_bluet potted_bamboo potted_birch_sapling potted_blue_orchid potted_brown_mushroom potted_cactus potted_cherry_sapling potted_closed_eyeblossom potted_cornflower potted_crimson_fungus potted_crimson_roots potted_dandelion potted_dark_oak_sapling potted_dead_bush potted_fern potted_flowering_azalea_bush potted_golden_dandelion potted_jungle_sapling potted_lily_of_the_valley potted_mangrove_propagule potted_oak_sapling potted_open_eyeblossom potted_orange_tulip potted_oxeye_daisy potted_pale_oak_sapling potted_pink_tulip potted_poplar_sapling potted_poppy potted_red_mushroom potted_red_tulip potted_spruce_sapling potted_torchflower potted_warped_fungus potted_warped_roots potted_white_tulip potted_wither_rose",
    ),
    (
        "minecraft:flowers",
        "allium azure_bluet blue_orchid cactus_flower cherry_leaves chorus_flower closed_eyeblossom cornflower dandelion flowering_azalea flowering_azalea_leaves golden_dandelion lilac lily_of_the_valley mangrove_propagule open_eyeblossom orange_tulip oxeye_daisy peony pink_petals pink_tulip pitcher_plant poppy red_tulip rose_bush spore_blossom sunflower torchflower white_tulip wildflowers wither_rose",
    ),
    (
        "minecraft:forest_rock_can_place_on",
        "andesite coarse_dirt deepslate diorite dirt granite grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt stone tuff",
    ),
    ("minecraft:fox_immune_to", "sweet_berry_bush"),
    (
        "minecraft:foxes_spawnable_on",
        "coarse_dirt grass_block podzol snow snow_block",
    ),
    ("minecraft:frog_prefer_jump_to", "big_dripleaf lily_pad"),
    (
        "minecraft:frogs_spawnable_on",
        "grass_block mangrove_roots mud muddy_mangrove_roots",
    ),
    (
        "minecraft:geode_invalid_blocks",
        "bedrock blue_ice ice lava packed_ice water",
    ),
    (
        "minecraft:glazed_terracotta",
        "black_glazed_terracotta blue_glazed_terracotta brown_glazed_terracotta cyan_glazed_terracotta gray_glazed_terracotta green_glazed_terracotta light_blue_glazed_terracotta light_gray_glazed_terracotta lime_glazed_terracotta magenta_glazed_terracotta orange_glazed_terracotta pink_glazed_terracotta purple_glazed_terracotta red_glazed_terracotta white_glazed_terracotta yellow_glazed_terracotta",
    ),
    (
        "minecraft:goats_spawnable_on",
        "grass_block gravel packed_ice snow snow_block stone",
    ),
    (
        "minecraft:gold_ores",
        "deepslate_gold_ore gold_ore nether_gold_ore",
    ),
    ("minecraft:grass_blocks", "grass_block mycelium podzol"),
    ("minecraft:grows_crops", "farmland"),
    (
        "minecraft:guarded_by_piglins",
        "barrel black_shulker_box blue_shulker_box brown_shulker_box chest copper_chest cyan_shulker_box deepslate_gold_ore ender_chest exposed_copper_chest gilded_blackstone gold_block gold_ore gray_shulker_box green_shulker_box light_blue_shulker_box light_gray_shulker_box lime_shulker_box magenta_shulker_box nether_gold_ore orange_shulker_box oxidized_copper_chest pink_shulker_box purple_shulker_box raw_gold_block red_shulker_box shulker_box trapped_chest waxed_copper_chest waxed_exposed_copper_chest waxed_oxidized_copper_chest waxed_weathered_copper_chest weathered_copper_chest white_shulker_box yellow_shulker_box",
    ),
    (
        "minecraft:happy_ghast_avoids",
        "cactus fire magma_block pointed_dripstone sulfur_spike sweet_berry_bush wither_rose",
    ),
    ("minecraft:height_specific_ore_replaceables", "tuff"),
    (
        "minecraft:hoglin_repellents",
        "nether_portal potted_warped_fungus respawn_anchor warped_fungus",
    ),
    (
        "minecraft:huge_brown_mushroom_can_place_on",
        "coarse_dirt crimson_nylium dirt grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt warped_nylium",
    ),
    (
        "minecraft:huge_red_mushroom_can_place_on",
        "coarse_dirt crimson_nylium dirt grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt warped_nylium",
    ),
    ("minecraft:ice", "blue_ice frosted_ice ice packed_ice"),
    (
        "minecraft:ice_melts_when_destroyed_above",
        "acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_leaves acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wood amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil azalea_leaves bamboo bamboo_block bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign barrel barrier basalt beacon bedrock bee_nest beehive bell birch_door birch_fence birch_fence_gate birch_hanging_sign birch_leaves birch_log birch_planks birch_pressure_plate birch_shelf birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wood black_banner black_bed black_candle_cake black_concrete black_concrete_powder black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wall_banner black_wool black_wool_slab black_wool_stairs blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_banner blue_bed blue_candle_cake blue_concrete blue_concrete_powder blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wall_banner blue_wool blue_wool_slab blue_wool_stairs bone_block bookshelf brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_banner brown_bed brown_candle_cake brown_concrete brown_concrete_powder brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_mushroom_block brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wall_banner brown_wool brown_wool_slab brown_wool_stairs bubble_coral_block budding_amethyst cactus cake calcite calibrated_sculk_sensor campfire candle_cake cartography_table carved_pumpkin cauldron chain_command_block cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_leaves cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wood chest chipped_anvil chiseled_bookshelf chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall clay coal_block coal_ore coarse_dirt cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall command_block composter conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crafting_table creaking_heart crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_nylium crimson_planks crimson_pressure_plate crimson_shelf crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_banner cyan_bed cyan_candle_cake cyan_concrete cyan_concrete_powder cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wall_banner cyan_wool cyan_wool_slab cyan_wool_stairs damaged_anvil dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_leaves dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wood dark_prismarine dark_prismarine_slab dark_prismarine_stairs daylight_detector dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan decorated_pot deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dirt dirt_path dispenser dragon_egg dried_ghast dried_kelp_block dripstone_block dropper emerald_block emerald_ore enchanting_table end_portal_frame end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod farmland fire_coral_block fletching_table flowering_azalea_leaves frosted_ice furnace gilded_blackstone glass glass_pane glowstone gold_block gold_ore granite granite_slab granite_stairs granite_wall grass_block gravel gray_banner gray_bed gray_candle_cake gray_concrete gray_concrete_powder gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wall_banner gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_candle_cake green_concrete green_concrete_powder green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wall_banner green_wool green_wool_slab green_wool_stairs grindstone hay_block heavy_weighted_pressure_plate honey_block honeycomb_block hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor jack_o_lantern jigsaw jukebox jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_leaves jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wood lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron lectern light_blue_banner light_blue_bed light_blue_candle_cake light_blue_concrete light_blue_concrete_powder light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wall_banner light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_candle_cake light_gray_concrete light_gray_concrete_powder light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wall_banner light_gray_wool light_gray_wool_slab light_gray_wool_stairs light_weighted_pressure_plate lightning_rod lime_banner lime_bed lime_candle_cake lime_concrete lime_concrete_powder lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wall_banner lime_wool lime_wool_slab lime_wool_stairs lodestone loom magenta_banner magenta_bed magenta_candle_cake magenta_concrete magenta_concrete_powder magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wall_banner magenta_wool magenta_wool_slab magenta_wool_stairs magma_block mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_leaves mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wood medium_amethyst_bud melon moss_block mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks moving_piston mud mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack note_block oak_door oak_fence oak_fence_gate oak_hanging_sign oak_leaves oak_log oak_planks oak_pressure_plate oak_shelf oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wood observer obsidian ochre_froglight orange_banner orange_bed orange_candle_cake orange_concrete orange_concrete_powder orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_poplar_leaves orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wall_banner orange_wool orange_wool_slab orange_wool_stairs oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud pale_moss_block pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_leaves pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wood pearlescent_froglight petrified_oak_slab pink_banner pink_bed pink_candle_cake pink_concrete pink_concrete_powder pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wall_banner pink_wool pink_wool_slab pink_wool_stairs piston piston_head podzol pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wood potent_sulfur powder_snow_cauldron prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall pumpkin purple_banner purple_bed purple_candle_cake purple_concrete purple_concrete_powder purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wall_banner purple_wool purple_wool_slab purple_wool_stairs purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs raw_copper_block raw_gold_block raw_iron_block red_banner red_bed red_candle_cake red_concrete red_concrete_powder red_concrete_slab red_concrete_stairs red_glazed_terracotta red_mushroom_block red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_poplar_leaves red_sand red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wall_banner red_wool red_wool_slab red_wool_stairs redstone_block redstone_lamp redstone_ore reinforced_deepslate repeating_command_block resin_block resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor rooted_dirt sand sandstone sandstone_slab sandstone_stairs sandstone_wall sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein sea_lantern shroomlight shulker_box slime_block small_amethyst_bud smithing_table smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab sniffer_egg snow_block soul_campfire soul_lantern soul_sand soul_soil spawner sponge spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_leaves spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wood sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_pressure_plate stone_slab stone_stairs stonecutter straw_bed stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem structure_block sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall suspicious_gravel suspicious_sand target terracotta test_block test_instance_block tinted_glass tnt trapped_chest trial_spawner tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall turtle_egg vault verdant_froglight warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_nylium warped_planks warped_pressure_plate warped_shelf warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wart_block water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod wet_sponge white_banner white_bed white_candle_cake white_concrete white_concrete_powder white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wall_banner white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_candle_cake yellow_concrete yellow_concrete_powder yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_poplar_leaves yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wall_banner yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:ice_spike_replaceable",
        "coarse_dirt dirt grass_block ice moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt snow_block",
    ),
    (
        "minecraft:impermeable",
        "barrier black_stained_glass blue_stained_glass brown_stained_glass cyan_stained_glass glass gray_stained_glass green_stained_glass light_blue_stained_glass light_gray_stained_glass lime_stained_glass magenta_stained_glass orange_stained_glass pink_stained_glass purple_stained_glass red_stained_glass tinted_glass white_stained_glass yellow_stained_glass",
    ),
    (
        "minecraft:incorrect_for_copper_tool",
        "ancient_debris crying_obsidian deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_redstone_ore diamond_block diamond_ore emerald_block emerald_ore gold_block gold_ore netherite_block obsidian raw_gold_block redstone_ore respawn_anchor",
    ),
    ("minecraft:incorrect_for_diamond_tool", ""),
    (
        "minecraft:incorrect_for_gold_tool",
        "ancient_debris chiseled_copper copper_block copper_bulb copper_chest copper_grate copper_ore copper_trapdoor crafter crying_obsidian cut_copper cut_copper_slab cut_copper_stairs deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore diamond_block diamond_ore emerald_block emerald_ore exposed_chiseled_copper exposed_copper exposed_copper_bulb exposed_copper_chest exposed_copper_grate exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod gold_block gold_ore iron_block iron_ore lapis_block lapis_ore lightning_rod netherite_block obsidian oxidized_chiseled_copper oxidized_copper oxidized_copper_bulb oxidized_copper_chest oxidized_copper_grate oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod raw_copper_block raw_gold_block raw_iron_block redstone_ore respawn_anchor waxed_chiseled_copper waxed_copper_block waxed_copper_bulb waxed_copper_chest waxed_copper_grate waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bulb waxed_exposed_copper_chest waxed_exposed_copper_grate waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bulb waxed_oxidized_copper_chest waxed_oxidized_copper_grate waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bulb waxed_weathered_copper_chest waxed_weathered_copper_grate waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bulb weathered_copper_chest weathered_copper_grate weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod",
    ),
    (
        "minecraft:incorrect_for_iron_tool",
        "ancient_debris crying_obsidian netherite_block obsidian respawn_anchor",
    ),
    ("minecraft:incorrect_for_netherite_tool", ""),
    (
        "minecraft:incorrect_for_stone_tool",
        "ancient_debris crying_obsidian deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_redstone_ore diamond_block diamond_ore emerald_block emerald_ore gold_block gold_ore netherite_block obsidian raw_gold_block redstone_ore respawn_anchor",
    ),
    (
        "minecraft:incorrect_for_wooden_tool",
        "ancient_debris chiseled_copper copper_block copper_bulb copper_chest copper_grate copper_ore copper_trapdoor crafter crying_obsidian cut_copper cut_copper_slab cut_copper_stairs deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore diamond_block diamond_ore emerald_block emerald_ore exposed_chiseled_copper exposed_copper exposed_copper_bulb exposed_copper_chest exposed_copper_grate exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod gold_block gold_ore iron_block iron_ore lapis_block lapis_ore lightning_rod netherite_block obsidian oxidized_chiseled_copper oxidized_copper oxidized_copper_bulb oxidized_copper_chest oxidized_copper_grate oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod raw_copper_block raw_gold_block raw_iron_block redstone_ore respawn_anchor waxed_chiseled_copper waxed_copper_block waxed_copper_bulb waxed_copper_chest waxed_copper_grate waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bulb waxed_exposed_copper_chest waxed_exposed_copper_grate waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bulb waxed_oxidized_copper_chest waxed_oxidized_copper_grate waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bulb waxed_weathered_copper_chest waxed_weathered_copper_grate waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bulb weathered_copper_chest weathered_copper_grate weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod",
    ),
    ("minecraft:infiniburn_end", "bedrock magma_block netherrack"),
    ("minecraft:infiniburn_nether", "magma_block netherrack"),
    ("minecraft:infiniburn_overworld", "magma_block netherrack"),
    (
        "minecraft:inside_step_sound_blocks",
        "glow_lichen leaf_litter lily_pad pink_petals powder_snow sculk_vein small_amethyst_bud wildflowers",
    ),
    ("minecraft:invalid_spawn_inside", "end_gateway end_portal"),
    ("minecraft:iron_ores", "deepslate_iron_ore iron_ore"),
    (
        "minecraft:jungle_logs",
        "jungle_log jungle_wood stripped_jungle_log stripped_jungle_wood",
    ),
    (
        "minecraft:lanterns",
        "copper_lantern exposed_copper_lantern lantern oxidized_copper_lantern soul_lantern waxed_copper_lantern waxed_exposed_copper_lantern waxed_oxidized_copper_lantern waxed_weathered_copper_lantern weathered_copper_lantern",
    ),
    ("minecraft:lapis_ores", "deepslate_lapis_ore lapis_ore"),
    (
        "minecraft:lava_pool_stone_cannot_replace",
        "acacia_leaves acacia_log acacia_wood azalea_leaves bedrock birch_leaves birch_log birch_wood cherry_leaves cherry_log cherry_wood chest crimson_hyphae crimson_stem dark_oak_leaves dark_oak_log dark_oak_wood end_portal_frame flowering_azalea_leaves jungle_leaves jungle_log jungle_wood mangrove_leaves mangrove_log mangrove_wood oak_leaves oak_log oak_wood orange_poplar_leaves pale_oak_leaves pale_oak_log pale_oak_wood poplar_log poplar_wood red_poplar_leaves reinforced_deepslate spawner spruce_leaves spruce_log spruce_wood stripped_acacia_log stripped_acacia_wood stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem trial_spawner vault warped_hyphae warped_stem yellow_poplar_leaves",
    ),
    (
        "minecraft:leaves",
        "acacia_leaves azalea_leaves birch_leaves cherry_leaves dark_oak_leaves flowering_azalea_leaves jungle_leaves mangrove_leaves oak_leaves orange_poplar_leaves pale_oak_leaves red_poplar_leaves spruce_leaves yellow_poplar_leaves",
    ),
    (
        "minecraft:lightning_rods",
        "exposed_lightning_rod lightning_rod oxidized_lightning_rod waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_lightning_rod waxed_weathered_lightning_rod weathered_lightning_rod",
    ),
    (
        "minecraft:logs",
        "acacia_log acacia_wood birch_log birch_wood cherry_log cherry_wood crimson_hyphae crimson_stem dark_oak_log dark_oak_wood jungle_log jungle_wood mangrove_log mangrove_wood oak_log oak_wood pale_oak_log pale_oak_wood poplar_log poplar_wood spruce_log spruce_wood stripped_acacia_log stripped_acacia_wood stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem warped_hyphae warped_stem",
    ),
    (
        "minecraft:logs_that_burn",
        "acacia_log acacia_wood birch_log birch_wood cherry_log cherry_wood dark_oak_log dark_oak_wood jungle_log jungle_wood mangrove_log mangrove_wood oak_log oak_wood pale_oak_log pale_oak_wood poplar_log poplar_wood spruce_log spruce_wood stripped_acacia_log stripped_acacia_wood stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood",
    ),
    (
        "minecraft:lush_ground_replaceable",
        "andesite cave_vines cave_vines_plant clay coarse_dirt deepslate diorite dirt granite grass_block gravel moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt sand stone tuff",
    ),
    (
        "minecraft:maintains_farmland",
        "acacia_fence_gate attached_melon_stem attached_pumpkin_stem bamboo_fence_gate beetroots birch_fence_gate carrots cherry_fence_gate crimson_fence_gate dark_oak_fence_gate jungle_fence_gate mangrove_fence_gate melon_stem moving_piston oak_fence_gate pale_oak_fence_gate pitcher_crop poplar_fence_gate potatoes pumpkin_stem spruce_fence_gate torchflower torchflower_crop warped_fence_gate wheat",
    ),
    (
        "minecraft:mangrove_logs",
        "mangrove_log mangrove_wood stripped_mangrove_log stripped_mangrove_wood",
    ),
    (
        "minecraft:mangrove_logs_can_grow_through",
        "mangrove_leaves mangrove_log mangrove_propagule mangrove_roots moss_carpet mud muddy_mangrove_roots vine",
    ),
    (
        "minecraft:mangrove_roots_can_grow_through",
        "mangrove_propagule mangrove_roots moss_carpet mud muddy_mangrove_roots snow vine",
    ),
    (
        "minecraft:mineable/axe",
        "acacia_button acacia_door acacia_fence acacia_fence_gate acacia_hanging_sign acacia_log acacia_planks acacia_pressure_plate acacia_shelf acacia_sign acacia_slab acacia_stairs acacia_trapdoor acacia_wall_hanging_sign acacia_wall_sign acacia_wood bamboo bamboo_block bamboo_button bamboo_door bamboo_fence bamboo_fence_gate bamboo_hanging_sign bamboo_mosaic bamboo_mosaic_slab bamboo_mosaic_stairs bamboo_planks bamboo_pressure_plate bamboo_shelf bamboo_sign bamboo_slab bamboo_stairs bamboo_trapdoor bamboo_wall_hanging_sign bamboo_wall_sign barrel bee_nest beehive big_dripleaf big_dripleaf_stem birch_button birch_door birch_fence birch_fence_gate birch_hanging_sign birch_log birch_planks birch_pressure_plate birch_shelf birch_sign birch_slab birch_stairs birch_trapdoor birch_wall_hanging_sign birch_wall_sign birch_wood black_banner black_wall_banner blue_banner blue_wall_banner bookshelf brown_banner brown_mushroom_block brown_wall_banner campfire cartography_table carved_pumpkin cherry_button cherry_door cherry_fence cherry_fence_gate cherry_hanging_sign cherry_log cherry_planks cherry_pressure_plate cherry_shelf cherry_sign cherry_slab cherry_stairs cherry_trapdoor cherry_wall_hanging_sign cherry_wall_sign cherry_wood chest chiseled_bookshelf chorus_flower chorus_plant cocoa composter crafting_table creaking_heart crimson_button crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_planks crimson_pressure_plate crimson_shelf crimson_sign crimson_slab crimson_stairs crimson_stem crimson_trapdoor crimson_wall_hanging_sign crimson_wall_sign cyan_banner cyan_wall_banner dark_oak_button dark_oak_door dark_oak_fence dark_oak_fence_gate dark_oak_hanging_sign dark_oak_log dark_oak_planks dark_oak_pressure_plate dark_oak_shelf dark_oak_sign dark_oak_slab dark_oak_stairs dark_oak_trapdoor dark_oak_wall_hanging_sign dark_oak_wall_sign dark_oak_wood daylight_detector fletching_table glow_lichen gray_banner gray_wall_banner green_banner green_wall_banner jack_o_lantern jukebox jungle_button jungle_door jungle_fence jungle_fence_gate jungle_hanging_sign jungle_log jungle_planks jungle_pressure_plate jungle_shelf jungle_sign jungle_slab jungle_stairs jungle_trapdoor jungle_wall_hanging_sign jungle_wall_sign jungle_wood ladder lectern light_blue_banner light_blue_wall_banner light_gray_banner light_gray_wall_banner lime_banner lime_wall_banner loom magenta_banner magenta_wall_banner mangrove_button mangrove_door mangrove_fence mangrove_fence_gate mangrove_hanging_sign mangrove_log mangrove_planks mangrove_pressure_plate mangrove_roots mangrove_shelf mangrove_sign mangrove_slab mangrove_stairs mangrove_trapdoor mangrove_wall_hanging_sign mangrove_wall_sign mangrove_wood melon mushroom_stem note_block oak_button oak_door oak_fence oak_fence_gate oak_hanging_sign oak_log oak_planks oak_pressure_plate oak_shelf oak_sign oak_slab oak_stairs oak_trapdoor oak_wall_hanging_sign oak_wall_sign oak_wood orange_banner orange_wall_banner pale_oak_button pale_oak_door pale_oak_fence pale_oak_fence_gate pale_oak_hanging_sign pale_oak_log pale_oak_planks pale_oak_pressure_plate pale_oak_shelf pale_oak_sign pale_oak_slab pale_oak_stairs pale_oak_trapdoor pale_oak_wall_hanging_sign pale_oak_wall_sign pale_oak_wood pink_banner pink_wall_banner poplar_button poplar_door poplar_fence poplar_fence_gate poplar_hanging_sign poplar_log poplar_planks poplar_pressure_plate poplar_shelf poplar_sign poplar_stairs poplar_trapdoor poplar_wall_hanging_sign poplar_wall_sign poplar_wood pumpkin purple_banner purple_wall_banner red_banner red_mushroom_block red_wall_banner smithing_table soul_campfire spruce_button spruce_door spruce_fence spruce_fence_gate spruce_hanging_sign spruce_log spruce_planks spruce_pressure_plate spruce_shelf spruce_sign spruce_slab spruce_stairs spruce_trapdoor spruce_wall_hanging_sign spruce_wall_sign spruce_wood stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem trapped_chest vine warped_button warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_planks warped_pressure_plate warped_shelf warped_sign warped_slab warped_stairs warped_stem warped_trapdoor warped_wall_hanging_sign warped_wall_sign white_banner white_wall_banner yellow_banner yellow_wall_banner",
    ),
    (
        "minecraft:mineable/hoe",
        "acacia_leaves azalea_leaves birch_leaves calibrated_sculk_sensor cherry_leaves dark_oak_leaves dried_kelp_block flowering_azalea_leaves hay_block jungle_leaves mangrove_leaves moss_block moss_carpet nether_wart_block oak_leaves orange_poplar_leaves pale_moss_block pale_moss_carpet pale_oak_leaves red_poplar_leaves sculk sculk_catalyst sculk_sensor sculk_shrieker sculk_vein shroomlight sponge spruce_leaves straw_bed target warped_wart_block wet_sponge yellow_poplar_leaves",
    ),
    (
        "minecraft:mineable/pickaxe",
        "activator_rail amethyst_block amethyst_cluster ancient_debris andesite andesite_slab andesite_stairs andesite_wall anvil basalt bell black_concrete black_concrete_slab black_concrete_stairs black_glazed_terracotta black_shulker_box black_terracotta blackstone blackstone_slab blackstone_stairs blackstone_wall blast_furnace blue_concrete blue_concrete_slab blue_concrete_stairs blue_glazed_terracotta blue_ice blue_shulker_box blue_terracotta bone_block brain_coral_block brewing_stand brick_slab brick_stairs brick_wall bricks brown_concrete brown_concrete_slab brown_concrete_stairs brown_glazed_terracotta brown_shulker_box brown_terracotta bubble_coral_block budding_amethyst calcite cauldron chipped_anvil chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_brick_slab cinnabar_brick_stairs cinnabar_brick_wall cinnabar_bricks cinnabar_slab cinnabar_stairs cinnabar_wall coal_block coal_ore cobbled_deepslate cobbled_deepslate_slab cobbled_deepslate_stairs cobbled_deepslate_wall cobblestone cobblestone_slab cobblestone_stairs cobblestone_wall conduit copper_bars copper_block copper_bulb copper_chain copper_chest copper_door copper_golem_statue copper_grate copper_lantern copper_ore copper_trapdoor cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crafter crimson_nylium crying_obsidian cut_copper cut_copper_slab cut_copper_stairs cut_red_sandstone cut_red_sandstone_slab cut_sandstone cut_sandstone_slab cyan_concrete cyan_concrete_slab cyan_concrete_stairs cyan_glazed_terracotta cyan_shulker_box cyan_terracotta damaged_anvil dark_prismarine dark_prismarine_slab dark_prismarine_stairs dead_brain_coral dead_brain_coral_block dead_brain_coral_fan dead_brain_coral_wall_fan dead_bubble_coral dead_bubble_coral_block dead_bubble_coral_fan dead_bubble_coral_wall_fan dead_fire_coral dead_fire_coral_block dead_fire_coral_fan dead_fire_coral_wall_fan dead_horn_coral dead_horn_coral_block dead_horn_coral_fan dead_horn_coral_wall_fan dead_tube_coral dead_tube_coral_block dead_tube_coral_fan dead_tube_coral_wall_fan deepslate deepslate_brick_slab deepslate_brick_stairs deepslate_brick_wall deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tile_slab deepslate_tile_stairs deepslate_tile_wall deepslate_tiles detector_rail diamond_block diamond_ore diorite diorite_slab diorite_stairs diorite_wall dispenser dripstone_block dropper emerald_block emerald_ore enchanting_table end_stone end_stone_brick_slab end_stone_brick_stairs end_stone_brick_wall end_stone_bricks ender_chest exposed_chiseled_copper exposed_copper exposed_copper_bars exposed_copper_bulb exposed_copper_chain exposed_copper_chest exposed_copper_door exposed_copper_golem_statue exposed_copper_grate exposed_copper_lantern exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod fire_coral_block furnace gilded_blackstone gold_block gold_ore granite granite_slab granite_stairs granite_wall gray_concrete gray_concrete_slab gray_concrete_stairs gray_glazed_terracotta gray_shulker_box gray_terracotta green_concrete green_concrete_slab green_concrete_stairs green_glazed_terracotta green_shulker_box green_terracotta grindstone heavy_core heavy_weighted_pressure_plate hopper horn_coral_block ice infested_chiseled_stone_bricks infested_cobblestone infested_cracked_stone_bricks infested_deepslate infested_mossy_stone_bricks infested_stone infested_stone_bricks iron_bars iron_block iron_chain iron_door iron_ore iron_trapdoor lantern lapis_block lapis_ore large_amethyst_bud lava_cauldron light_blue_concrete light_blue_concrete_slab light_blue_concrete_stairs light_blue_glazed_terracotta light_blue_shulker_box light_blue_terracotta light_gray_concrete light_gray_concrete_slab light_gray_concrete_stairs light_gray_glazed_terracotta light_gray_shulker_box light_gray_terracotta light_weighted_pressure_plate lightning_rod lime_concrete lime_concrete_slab lime_concrete_stairs lime_glazed_terracotta lime_shulker_box lime_terracotta lodestone magenta_concrete magenta_concrete_slab magenta_concrete_stairs magenta_glazed_terracotta magenta_shulker_box magenta_terracotta magma_block medium_amethyst_bud mossy_cobblestone mossy_cobblestone_slab mossy_cobblestone_stairs mossy_cobblestone_wall mossy_stone_brick_slab mossy_stone_brick_stairs mossy_stone_brick_wall mossy_stone_bricks mud_brick_slab mud_brick_stairs mud_brick_wall mud_bricks nether_brick_fence nether_brick_slab nether_brick_stairs nether_brick_wall nether_bricks nether_gold_ore nether_quartz_ore netherite_block netherrack observer obsidian orange_concrete orange_concrete_slab orange_concrete_stairs orange_glazed_terracotta orange_shulker_box orange_terracotta oxidized_chiseled_copper oxidized_copper oxidized_copper_bars oxidized_copper_bulb oxidized_copper_chain oxidized_copper_chest oxidized_copper_door oxidized_copper_golem_statue oxidized_copper_grate oxidized_copper_lantern oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod packed_ice packed_mud petrified_oak_slab pink_concrete pink_concrete_slab pink_concrete_stairs pink_glazed_terracotta pink_shulker_box pink_terracotta piston piston_head pointed_dripstone polished_andesite polished_andesite_slab polished_andesite_stairs polished_basalt polished_blackstone polished_blackstone_brick_slab polished_blackstone_brick_stairs polished_blackstone_brick_wall polished_blackstone_bricks polished_blackstone_button polished_blackstone_pressure_plate polished_blackstone_slab polished_blackstone_stairs polished_blackstone_wall polished_cinnabar polished_cinnabar_slab polished_cinnabar_stairs polished_cinnabar_wall polished_deepslate polished_deepslate_slab polished_deepslate_stairs polished_deepslate_wall polished_diorite polished_diorite_slab polished_diorite_stairs polished_granite polished_granite_slab polished_granite_stairs polished_sulfur polished_sulfur_slab polished_sulfur_stairs polished_sulfur_wall polished_tuff polished_tuff_slab polished_tuff_stairs polished_tuff_wall potent_sulfur powder_snow_cauldron powered_rail prismarine prismarine_brick_slab prismarine_brick_stairs prismarine_bricks prismarine_slab prismarine_stairs prismarine_wall purple_concrete purple_concrete_slab purple_concrete_stairs purple_glazed_terracotta purple_shulker_box purple_terracotta purpur_block purpur_pillar purpur_slab purpur_stairs quartz_block quartz_bricks quartz_pillar quartz_slab quartz_stairs rail raw_copper_block raw_gold_block raw_iron_block red_concrete red_concrete_slab red_concrete_stairs red_glazed_terracotta red_nether_brick_slab red_nether_brick_stairs red_nether_brick_wall red_nether_bricks red_sandstone red_sandstone_slab red_sandstone_stairs red_sandstone_wall red_shulker_box red_terracotta redstone_block redstone_ore resin_brick_slab resin_brick_stairs resin_brick_wall resin_bricks respawn_anchor sandstone sandstone_slab sandstone_stairs sandstone_wall shulker_box small_amethyst_bud smoker smooth_basalt smooth_quartz smooth_quartz_slab smooth_quartz_stairs smooth_red_sandstone smooth_red_sandstone_slab smooth_red_sandstone_stairs smooth_sandstone smooth_sandstone_slab smooth_sandstone_stairs smooth_stone smooth_stone_slab soul_lantern spawner sticky_piston stone stone_brick_slab stone_brick_stairs stone_brick_wall stone_bricks stone_button stone_pressure_plate stone_slab stone_stairs stonecutter sulfur sulfur_brick_slab sulfur_brick_stairs sulfur_brick_wall sulfur_bricks sulfur_slab sulfur_spike sulfur_stairs sulfur_wall terracotta tube_coral_block tuff tuff_brick_slab tuff_brick_stairs tuff_brick_wall tuff_bricks tuff_slab tuff_stairs tuff_wall warped_nylium water_cauldron waxed_chiseled_copper waxed_copper_bars waxed_copper_block waxed_copper_bulb waxed_copper_chain waxed_copper_chest waxed_copper_door waxed_copper_golem_statue waxed_copper_grate waxed_copper_lantern waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bars waxed_exposed_copper_bulb waxed_exposed_copper_chain waxed_exposed_copper_chest waxed_exposed_copper_door waxed_exposed_copper_golem_statue waxed_exposed_copper_grate waxed_exposed_copper_lantern waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bars waxed_oxidized_copper_bulb waxed_oxidized_copper_chain waxed_oxidized_copper_chest waxed_oxidized_copper_door waxed_oxidized_copper_golem_statue waxed_oxidized_copper_grate waxed_oxidized_copper_lantern waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bars waxed_weathered_copper_bulb waxed_weathered_copper_chain waxed_weathered_copper_chest waxed_weathered_copper_door waxed_weathered_copper_golem_statue waxed_weathered_copper_grate waxed_weathered_copper_lantern waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bars weathered_copper_bulb weathered_copper_chain weathered_copper_chest weathered_copper_door weathered_copper_golem_statue weathered_copper_grate weathered_copper_lantern weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod white_concrete white_concrete_slab white_concrete_stairs white_glazed_terracotta white_shulker_box white_terracotta yellow_concrete yellow_concrete_slab yellow_concrete_stairs yellow_glazed_terracotta yellow_shulker_box yellow_terracotta",
    ),
    (
        "minecraft:mineable/shovel",
        "black_concrete_powder blue_concrete_powder brown_concrete_powder clay coarse_dirt cyan_concrete_powder dirt dirt_path farmland grass_block gravel gray_concrete_powder green_concrete_powder light_blue_concrete_powder light_gray_concrete_powder lime_concrete_powder magenta_concrete_powder mud muddy_mangrove_roots mycelium orange_concrete_powder pink_concrete_powder podzol purple_concrete_powder red_concrete_powder red_sand rooted_dirt sand snow snow_block soul_sand soul_soil suspicious_gravel suspicious_sand white_concrete_powder yellow_concrete_powder",
    ),
    (
        "minecraft:mob_interactable_doors",
        "acacia_door bamboo_door birch_door cherry_door copper_door crimson_door dark_oak_door exposed_copper_door jungle_door mangrove_door oak_door oxidized_copper_door pale_oak_door poplar_door spruce_door warped_door waxed_copper_door waxed_exposed_copper_door waxed_oxidized_copper_door waxed_weathered_copper_door weathered_copper_door",
    ),
    ("minecraft:mooshrooms_spawnable_on", "mycelium"),
    ("minecraft:moss_blocks", "moss_block pale_moss_block"),
    (
        "minecraft:moss_replaceable",
        "andesite cave_vines cave_vines_plant coarse_dirt deepslate diorite dirt granite grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt stone tuff",
    ),
    ("minecraft:mud", "mud muddy_mangrove_roots"),
    (
        "minecraft:needs_diamond_tool",
        "ancient_debris crying_obsidian netherite_block obsidian respawn_anchor",
    ),
    (
        "minecraft:needs_iron_tool",
        "deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_redstone_ore diamond_block diamond_ore emerald_block emerald_ore gold_block gold_ore raw_gold_block redstone_ore",
    ),
    (
        "minecraft:needs_stone_tool",
        "chiseled_copper copper_block copper_bulb copper_chest copper_grate copper_ore copper_trapdoor crafter cut_copper cut_copper_slab cut_copper_stairs deepslate_copper_ore deepslate_iron_ore deepslate_lapis_ore exposed_chiseled_copper exposed_copper exposed_copper_bulb exposed_copper_chest exposed_copper_grate exposed_copper_trapdoor exposed_cut_copper exposed_cut_copper_slab exposed_cut_copper_stairs exposed_lightning_rod iron_block iron_ore lapis_block lapis_ore lightning_rod oxidized_chiseled_copper oxidized_copper oxidized_copper_bulb oxidized_copper_chest oxidized_copper_grate oxidized_copper_trapdoor oxidized_cut_copper oxidized_cut_copper_slab oxidized_cut_copper_stairs oxidized_lightning_rod raw_copper_block raw_iron_block waxed_chiseled_copper waxed_copper_block waxed_copper_bulb waxed_copper_chest waxed_copper_grate waxed_copper_trapdoor waxed_cut_copper waxed_cut_copper_slab waxed_cut_copper_stairs waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bulb waxed_exposed_copper_chest waxed_exposed_copper_grate waxed_exposed_copper_trapdoor waxed_exposed_cut_copper waxed_exposed_cut_copper_slab waxed_exposed_cut_copper_stairs waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bulb waxed_oxidized_copper_chest waxed_oxidized_copper_grate waxed_oxidized_copper_trapdoor waxed_oxidized_cut_copper waxed_oxidized_cut_copper_slab waxed_oxidized_cut_copper_stairs waxed_oxidized_lightning_rod waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bulb waxed_weathered_copper_chest waxed_weathered_copper_grate waxed_weathered_copper_trapdoor waxed_weathered_cut_copper waxed_weathered_cut_copper_slab waxed_weathered_cut_copper_stairs waxed_weathered_lightning_rod weathered_chiseled_copper weathered_copper weathered_copper_bulb weathered_copper_chest weathered_copper_grate weathered_copper_trapdoor weathered_cut_copper weathered_cut_copper_slab weathered_cut_copper_stairs weathered_lightning_rod",
    ),
    ("minecraft:nether_portal_frame", "obsidian"),
    ("minecraft:nylium", "crimson_nylium warped_nylium"),
    (
        "minecraft:oak_logs",
        "oak_log oak_wood stripped_oak_log stripped_oak_wood",
    ),
    (
        "minecraft:occludes_vibration_signals",
        "black_wool blue_wool brown_wool cyan_wool gray_wool green_wool light_blue_wool light_gray_wool lime_wool magenta_wool orange_wool pink_wool purple_wool red_wool white_wool yellow_wool",
    ),
    (
        "minecraft:ores",
        "coal_ore copper_ore deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore diamond_ore emerald_ore gold_ore iron_ore lapis_ore nether_gold_ore nether_quartz_ore redstone_ore",
    ),
    (
        "minecraft:overrides_mushroom_light_requirement",
        "crimson_nylium mycelium podzol warped_nylium",
    ),
    (
        "minecraft:overworld_natural_logs",
        "acacia_log birch_log cherry_log dark_oak_log jungle_log mangrove_log oak_log pale_oak_log poplar_log spruce_log",
    ),
    (
        "minecraft:pale_oak_logs",
        "pale_oak_log pale_oak_wood stripped_pale_oak_log stripped_pale_oak_wood",
    ),
    (
        "minecraft:parrots_spawnable_on",
        "acacia_leaves acacia_log acacia_wood air azalea_leaves birch_leaves birch_log birch_wood cherry_leaves cherry_log cherry_wood crimson_hyphae crimson_stem dark_oak_leaves dark_oak_log dark_oak_wood flowering_azalea_leaves grass_block jungle_leaves jungle_log jungle_wood mangrove_leaves mangrove_log mangrove_wood oak_leaves oak_log oak_wood orange_poplar_leaves pale_oak_leaves pale_oak_log pale_oak_wood poplar_log poplar_wood red_poplar_leaves spruce_leaves spruce_log spruce_wood stripped_acacia_log stripped_acacia_wood stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem warped_hyphae warped_stem yellow_poplar_leaves",
    ),
    (
        "minecraft:piglin_repellents",
        "soul_campfire soul_fire soul_lantern soul_torch soul_wall_torch",
    ),
    (
        "minecraft:planks",
        "acacia_planks bamboo_planks birch_planks cherry_planks crimson_planks dark_oak_planks jungle_planks mangrove_planks oak_planks pale_oak_planks poplar_planks spruce_planks warped_planks",
    ),
    ("minecraft:polar_bear_immune_to", "powder_snow"),
    ("minecraft:polar_bears_spawnable_on_alternate", "ice"),
    (
        "minecraft:poplar_logs",
        "poplar_log poplar_wood stripped_poplar_log stripped_poplar_wood",
    ),
    ("minecraft:portals", "end_gateway end_portal nether_portal"),
    (
        "minecraft:pressure_plates",
        "acacia_pressure_plate bamboo_pressure_plate birch_pressure_plate cherry_pressure_plate crimson_pressure_plate dark_oak_pressure_plate heavy_weighted_pressure_plate jungle_pressure_plate light_weighted_pressure_plate mangrove_pressure_plate oak_pressure_plate pale_oak_pressure_plate polished_blackstone_pressure_plate poplar_pressure_plate spruce_pressure_plate stone_pressure_plate warped_pressure_plate",
    ),
    (
        "minecraft:prevent_mob_spawning_inside",
        "activator_rail detector_rail powered_rail rail",
    ),
    (
        "minecraft:prevents_nearby_leaf_decay",
        "acacia_log acacia_wood birch_log birch_wood cherry_log cherry_wood crimson_hyphae crimson_stem dark_oak_log dark_oak_wood jungle_log jungle_wood mangrove_log mangrove_wood oak_log oak_wood pale_oak_log pale_oak_wood poplar_log poplar_wood spruce_log spruce_wood stripped_acacia_log stripped_acacia_wood stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem warped_hyphae warped_stem",
    ),
    (
        "minecraft:rabbits_spawnable_on",
        "grass_block sand snow snow_block",
    ),
    (
        "minecraft:rails",
        "activator_rail detector_rail powered_rail rail",
    ),
    ("minecraft:redstone_ores", "deepslate_redstone_ore redstone_ore"),
    (
        "minecraft:replaceable",
        "air bubble_column bush cave_air crimson_roots dead_bush fern fire glow_lichen hanging_roots large_fern lava leaf_litter light nether_sprouts red_shrub resin_clump seagrass short_dry_grass short_grass snow soul_fire structure_void tall_dry_grass tall_grass tall_seagrass vine void_air warped_roots water",
    ),
    (
        "minecraft:replaceable_by_mushrooms",
        "acacia_leaves allium azalea_leaves azure_bluet birch_leaves blue_orchid brown_mushroom brown_mushroom_block bush cherry_leaves closed_eyeblossom cornflower crimson_roots dandelion dark_oak_leaves dead_bush fern firefly_bush flowering_azalea_leaves glow_lichen golden_dandelion hanging_roots jungle_leaves large_fern leaf_litter lilac lily_of_the_valley mangrove_leaves nether_sprouts oak_leaves open_eyeblossom orange_poplar_leaves orange_tulip oxeye_daisy pale_moss_carpet pale_oak_leaves peony pink_tulip pitcher_plant poppy red_mushroom red_mushroom_block red_poplar_leaves red_shrub red_tulip rose_bush seagrass short_dry_grass short_grass spruce_leaves sunflower tall_dry_grass tall_grass tall_seagrass torchflower vine warped_roots water white_tulip wither_rose yellow_poplar_leaves",
    ),
    (
        "minecraft:replaceable_by_trees",
        "acacia_leaves allium azalea_leaves azure_bluet birch_leaves blue_orchid bush cherry_leaves closed_eyeblossom cornflower crimson_roots dandelion dark_oak_leaves dead_bush fern firefly_bush flowering_azalea_leaves glow_lichen golden_dandelion hanging_roots jungle_leaves large_fern leaf_litter lilac lily_of_the_valley mangrove_leaves nether_sprouts oak_leaves open_eyeblossom orange_poplar_leaves orange_tulip oxeye_daisy pale_moss_carpet pale_oak_leaves peony pink_tulip pitcher_plant poppy red_poplar_leaves red_tulip rose_bush seagrass shelf_mushroom short_dry_grass short_grass spruce_leaves sunflower tall_dry_grass tall_grass tall_seagrass torchflower vine warped_roots water white_tulip wither_rose yellow_poplar_leaves",
    ),
    (
        "minecraft:required_for_poplar_leaf_ambience",
        "acacia_log birch_log cherry_log dark_oak_log jungle_log mangrove_log oak_log pale_oak_log poplar_log spruce_log",
    ),
    ("minecraft:sand", "red_sand sand suspicious_sand"),
    (
        "minecraft:saplings",
        "acacia_sapling azalea birch_sapling cherry_sapling dark_oak_sapling flowering_azalea jungle_sapling mangrove_propagule oak_sapling pale_oak_sapling poplar_sapling spruce_sapling",
    ),
    ("minecraft:sculk_growth_inhibitors", "sculk_sensor sculk_shrieker"),
    (
        "minecraft:sculk_replaceable",
        "andesite basalt black_terracotta blackstone blue_terracotta brown_terracotta calcite cinnabar clay coarse_dirt crimson_nylium cyan_terracotta deepslate diorite dirt dripstone_block end_stone granite grass_block gravel gray_terracotta green_terracotta light_blue_terracotta light_gray_terracotta lime_terracotta magenta_terracotta moss_block mud muddy_mangrove_roots mycelium netherrack orange_terracotta pale_moss_block pink_terracotta podzol purple_terracotta red_sand red_sandstone red_terracotta rooted_dirt sand sandstone smooth_basalt soul_sand soul_soil stone sulfur terracotta tuff warped_nylium white_terracotta yellow_terracotta",
    ),
    (
        "minecraft:sculk_replaceable_world_gen",
        "andesite basalt black_terracotta blackstone blue_terracotta brown_terracotta calcite cinnabar clay coarse_dirt cobbled_deepslate cracked_deepslate_bricks cracked_deepslate_tiles crimson_nylium cyan_terracotta deepslate deepslate_bricks deepslate_tiles diorite dirt dripstone_block end_stone granite grass_block gravel gray_terracotta green_terracotta light_blue_terracotta light_gray_terracotta lime_terracotta magenta_terracotta moss_block mud muddy_mangrove_roots mycelium netherrack orange_terracotta pale_moss_block pink_terracotta podzol polished_deepslate purple_terracotta red_sand red_sandstone red_terracotta rooted_dirt sand sandstone smooth_basalt soul_sand soul_soil stone sulfur terracotta tuff warped_nylium white_terracotta yellow_terracotta",
    ),
    (
        "minecraft:shears_extreme_breaking_speed",
        "acacia_leaves azalea_leaves birch_leaves cherry_leaves dark_oak_leaves flowering_azalea_leaves jungle_leaves mangrove_leaves oak_leaves orange_poplar_leaves pale_oak_leaves red_poplar_leaves spruce_leaves yellow_poplar_leaves",
    ),
    (
        "minecraft:shears_major_breaking_speed",
        "black_wool black_wool_slab black_wool_stairs blue_wool blue_wool_slab blue_wool_stairs brown_wool brown_wool_slab brown_wool_stairs cyan_wool cyan_wool_slab cyan_wool_stairs gray_wool gray_wool_slab gray_wool_stairs green_wool green_wool_slab green_wool_stairs light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_wool lime_wool_slab lime_wool_stairs magenta_wool magenta_wool_slab magenta_wool_stairs orange_wool orange_wool_slab orange_wool_stairs pink_wool pink_wool_slab pink_wool_stairs purple_wool purple_wool_slab purple_wool_stairs red_wool red_wool_slab red_wool_stairs white_wool white_wool_slab white_wool_stairs yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    ("minecraft:shears_minor_breaking_speed", "glow_lichen vine"),
    (
        "minecraft:shulker_boxes",
        "black_shulker_box blue_shulker_box brown_shulker_box cyan_shulker_box gray_shulker_box green_shulker_box light_blue_shulker_box light_gray_shulker_box lime_shulker_box magenta_shulker_box orange_shulker_box pink_shulker_box purple_shulker_box red_shulker_box shulker_box white_shulker_box yellow_shulker_box",
    ),
    ("minecraft:shulker_does_not_teleport_to", "bedrock"),
    (
        "minecraft:signs",
        "acacia_sign acacia_wall_sign bamboo_sign bamboo_wall_sign birch_sign birch_wall_sign cherry_sign cherry_wall_sign crimson_sign crimson_wall_sign dark_oak_sign dark_oak_wall_sign jungle_sign jungle_wall_sign mangrove_sign mangrove_wall_sign oak_sign oak_wall_sign pale_oak_sign pale_oak_wall_sign poplar_sign poplar_wall_sign spruce_sign spruce_wall_sign warped_sign warped_wall_sign",
    ),
    (
        "minecraft:skulls",
        "creeper_head dragon_head piglin_head player_head skeleton_skull wither_skeleton_skull zombie_head",
    ),
    (
        "minecraft:slabs",
        "acacia_slab andesite_slab bamboo_mosaic_slab bamboo_slab birch_slab black_concrete_slab black_wool_slab blackstone_slab blue_concrete_slab blue_wool_slab brick_slab brown_concrete_slab brown_wool_slab cherry_slab cinnabar_brick_slab cinnabar_slab cobbled_deepslate_slab cobblestone_slab crimson_slab cut_copper_slab cut_red_sandstone_slab cut_sandstone_slab cyan_concrete_slab cyan_wool_slab dark_oak_slab dark_prismarine_slab deepslate_brick_slab deepslate_tile_slab diorite_slab end_stone_brick_slab exposed_cut_copper_slab granite_slab gray_concrete_slab gray_wool_slab green_concrete_slab green_wool_slab jungle_slab light_blue_concrete_slab light_blue_wool_slab light_gray_concrete_slab light_gray_wool_slab lime_concrete_slab lime_wool_slab magenta_concrete_slab magenta_wool_slab mangrove_slab mossy_cobblestone_slab mossy_stone_brick_slab mud_brick_slab nether_brick_slab oak_slab orange_concrete_slab orange_wool_slab oxidized_cut_copper_slab pale_oak_slab petrified_oak_slab pink_concrete_slab pink_wool_slab polished_andesite_slab polished_blackstone_brick_slab polished_blackstone_slab polished_cinnabar_slab polished_deepslate_slab polished_diorite_slab polished_granite_slab polished_sulfur_slab polished_tuff_slab prismarine_brick_slab prismarine_slab purple_concrete_slab purple_wool_slab purpur_slab quartz_slab red_concrete_slab red_nether_brick_slab red_sandstone_slab red_wool_slab resin_brick_slab sandstone_slab smooth_quartz_slab smooth_red_sandstone_slab smooth_sandstone_slab smooth_stone_slab spruce_slab stone_brick_slab stone_slab sulfur_brick_slab sulfur_slab tuff_brick_slab tuff_slab warped_slab waxed_cut_copper_slab waxed_exposed_cut_copper_slab waxed_oxidized_cut_copper_slab waxed_weathered_cut_copper_slab weathered_cut_copper_slab white_concrete_slab white_wool_slab yellow_concrete_slab yellow_wool_slab",
    ),
    (
        "minecraft:small_flowers",
        "allium azure_bluet blue_orchid closed_eyeblossom cornflower dandelion golden_dandelion lily_of_the_valley open_eyeblossom orange_tulip oxeye_daisy pink_tulip poppy red_tulip torchflower white_tulip wither_rose",
    ),
    ("minecraft:smelts_to_glass", "red_sand sand"),
    (
        "minecraft:snaps_goat_horn",
        "acacia_log birch_log cherry_log coal_ore copper_ore dark_oak_log emerald_ore iron_ore jungle_log mangrove_log oak_log packed_ice pale_oak_log poplar_log spruce_log stone",
    ),
    (
        "minecraft:sniffer_diggable_block",
        "coarse_dirt dirt grass_block moss_block mud muddy_mangrove_roots pale_moss_block podzol rooted_dirt",
    ),
    ("minecraft:sniffer_egg_hatch_boost", "moss_block"),
    ("minecraft:snow", "powder_snow snow snow_block"),
    ("minecraft:snow_golem_immune_to", "powder_snow"),
    ("minecraft:soul_fire_base_blocks", "soul_sand soul_soil"),
    ("minecraft:soul_speed_blocks", "soul_sand soul_soil"),
    (
        "minecraft:speeds_up_zombie_villager_curing",
        "black_bed blue_bed brown_bed cyan_bed gray_bed green_bed iron_bars light_blue_bed light_gray_bed lime_bed magenta_bed orange_bed pink_bed purple_bed red_bed white_bed yellow_bed",
    ),
    ("minecraft:speleothems", "pointed_dripstone sulfur_spike"),
    (
        "minecraft:spruce_logs",
        "spruce_log spruce_wood stripped_spruce_log stripped_spruce_wood",
    ),
    (
        "minecraft:stairs",
        "acacia_stairs andesite_stairs bamboo_mosaic_stairs bamboo_stairs birch_stairs black_concrete_stairs black_wool_stairs blackstone_stairs blue_concrete_stairs blue_wool_stairs brick_stairs brown_concrete_stairs brown_wool_stairs cherry_stairs cinnabar_brick_stairs cinnabar_stairs cobbled_deepslate_stairs cobblestone_stairs crimson_stairs cut_copper_stairs cyan_concrete_stairs cyan_wool_stairs dark_oak_stairs dark_prismarine_stairs deepslate_brick_stairs deepslate_tile_stairs diorite_stairs end_stone_brick_stairs exposed_cut_copper_stairs granite_stairs gray_concrete_stairs gray_wool_stairs green_concrete_stairs green_wool_stairs jungle_stairs light_blue_concrete_stairs light_blue_wool_stairs light_gray_concrete_stairs light_gray_wool_stairs lime_concrete_stairs lime_wool_stairs magenta_concrete_stairs magenta_wool_stairs mangrove_stairs mossy_cobblestone_stairs mossy_stone_brick_stairs mud_brick_stairs nether_brick_stairs oak_stairs orange_concrete_stairs orange_wool_stairs oxidized_cut_copper_stairs pale_oak_stairs pink_concrete_stairs pink_wool_stairs polished_andesite_stairs polished_blackstone_brick_stairs polished_blackstone_stairs polished_cinnabar_stairs polished_deepslate_stairs polished_diorite_stairs polished_granite_stairs polished_sulfur_stairs polished_tuff_stairs poplar_stairs prismarine_brick_stairs prismarine_stairs purple_concrete_stairs purple_wool_stairs purpur_stairs quartz_stairs red_concrete_stairs red_nether_brick_stairs red_sandstone_stairs red_wool_stairs resin_brick_stairs sandstone_stairs smooth_quartz_stairs smooth_red_sandstone_stairs smooth_sandstone_stairs spruce_stairs stone_brick_stairs stone_stairs sulfur_brick_stairs sulfur_stairs tuff_brick_stairs tuff_stairs warped_stairs waxed_cut_copper_stairs waxed_exposed_cut_copper_stairs waxed_oxidized_cut_copper_stairs waxed_weathered_cut_copper_stairs weathered_cut_copper_stairs white_concrete_stairs white_wool_stairs yellow_concrete_stairs yellow_wool_stairs",
    ),
    (
        "minecraft:standing_signs",
        "acacia_sign bamboo_sign birch_sign cherry_sign crimson_sign dark_oak_sign jungle_sign mangrove_sign oak_sign pale_oak_sign poplar_sign spruce_sign warped_sign",
    ),
    (
        "minecraft:stone_bricks",
        "chiseled_stone_bricks cracked_stone_bricks mossy_stone_bricks stone_bricks",
    ),
    (
        "minecraft:stone_buttons",
        "polished_blackstone_button stone_button",
    ),
    (
        "minecraft:stone_ore_replaceables",
        "andesite diorite granite stone",
    ),
    (
        "minecraft:stone_pressure_plates",
        "polished_blackstone_pressure_plate stone_pressure_plate",
    ),
    ("minecraft:stray_immune_to", "powder_snow"),
    ("minecraft:strider_warm_blocks", "lava"),
    (
        "minecraft:substrate_overworld",
        "coarse_dirt dirt grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt",
    ),
    ("minecraft:sulfur_spike_replaceable_blocks", "cinnabar sulfur"),
    ("minecraft:support_override_cactus_flower", "cactus farmland"),
    (
        "minecraft:support_override_snow_layer",
        "honey_block mud soul_sand",
    ),
    (
        "minecraft:supports_azalea",
        "clay coarse_dirt dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt",
    ),
    (
        "minecraft:supports_bamboo",
        "bamboo bamboo_sapling coarse_dirt dirt grass_block gravel moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol red_sand rooted_dirt sand suspicious_gravel suspicious_sand",
    ),
    (
        "minecraft:supports_big_dripleaf",
        "clay coarse_dirt dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium podzol rooted_dirt",
    ),
    ("minecraft:supports_cactus", "red_sand sand suspicious_sand"),
    ("minecraft:supports_chorus_flower", "end_stone"),
    ("minecraft:supports_chorus_plant", "end_stone"),
    (
        "minecraft:supports_cocoa",
        "jungle_log jungle_wood stripped_jungle_log stripped_jungle_wood",
    ),
    (
        "minecraft:supports_crimson_fungus",
        "coarse_dirt crimson_nylium dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt soul_soil warped_nylium",
    ),
    (
        "minecraft:supports_crimson_roots",
        "coarse_dirt crimson_nylium dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt soul_soil warped_nylium",
    ),
    ("minecraft:supports_crops", "farmland"),
    (
        "minecraft:supports_dry_vegetation",
        "black_terracotta blue_terracotta brown_terracotta coarse_dirt cyan_terracotta dirt farmland grass_block gray_terracotta green_terracotta light_blue_terracotta light_gray_terracotta lime_terracotta magenta_terracotta moss_block mud muddy_mangrove_roots mycelium orange_terracotta pale_moss_block pink_terracotta podzol purple_terracotta red_sand red_terracotta rooted_dirt sand suspicious_sand terracotta white_terracotta yellow_terracotta",
    ),
    ("minecraft:supports_frogspawn", ""),
    ("minecraft:supports_hanging_mangrove_propagule", "mangrove_leaves"),
    ("minecraft:supports_lily_pad", "frosted_ice ice"),
    (
        "minecraft:supports_mangrove_propagule",
        "clay coarse_dirt dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt",
    ),
    ("minecraft:supports_melon_stem", "farmland"),
    (
        "minecraft:supports_melon_stem_fruit",
        "coarse_dirt dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt",
    ),
    (
        "minecraft:supports_nether_sprouts",
        "coarse_dirt crimson_nylium dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt soul_soil warped_nylium",
    ),
    ("minecraft:supports_nether_wart", "soul_sand"),
    ("minecraft:supports_pumpkin_stem", "farmland"),
    (
        "minecraft:supports_pumpkin_stem_fruit",
        "coarse_dirt dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt",
    ),
    ("minecraft:supports_small_dripleaf", "clay moss_block"),
    ("minecraft:supports_stem_crops", "farmland"),
    (
        "minecraft:supports_stem_fruit",
        "coarse_dirt dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt",
    ),
    (
        "minecraft:supports_sugar_cane",
        "coarse_dirt dirt grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol red_sand rooted_dirt sand suspicious_sand",
    ),
    ("minecraft:supports_sugar_cane_adjacently", "frosted_ice"),
    (
        "minecraft:supports_vegetation",
        "coarse_dirt dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt",
    ),
    (
        "minecraft:supports_warped_fungus",
        "coarse_dirt crimson_nylium dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt soul_soil warped_nylium",
    ),
    (
        "minecraft:supports_warped_roots",
        "coarse_dirt crimson_nylium dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium pale_moss_block podzol rooted_dirt soul_soil warped_nylium",
    ),
    (
        "minecraft:supports_wither_rose",
        "coarse_dirt dirt farmland grass_block moss_block mud muddy_mangrove_roots mycelium netherrack pale_moss_block podzol rooted_dirt soul_sand soul_soil",
    ),
    ("minecraft:suppresses_bounce", "honey_block"),
    (
        "minecraft:sword_efficient",
        "acacia_leaves azalea_leaves big_dripleaf big_dripleaf_stem birch_leaves carved_pumpkin cherry_leaves chorus_flower chorus_plant cocoa dark_oak_leaves flowering_azalea_leaves glow_lichen jack_o_lantern jungle_leaves mangrove_leaves melon oak_leaves orange_poplar_leaves pale_oak_leaves pumpkin red_poplar_leaves spruce_leaves vine yellow_poplar_leaves",
    ),
    ("minecraft:sword_instantly_mines", "bamboo bamboo_sapling"),
    (
        "minecraft:terracotta",
        "black_terracotta blue_terracotta brown_terracotta cyan_terracotta gray_terracotta green_terracotta light_blue_terracotta light_gray_terracotta lime_terracotta magenta_terracotta orange_terracotta pink_terracotta purple_terracotta red_terracotta terracotta white_terracotta yellow_terracotta",
    ),
    ("minecraft:trail_ruins_replaceable", "gravel"),
    (
        "minecraft:trapdoors",
        "acacia_trapdoor bamboo_trapdoor birch_trapdoor cherry_trapdoor copper_trapdoor crimson_trapdoor dark_oak_trapdoor exposed_copper_trapdoor iron_trapdoor jungle_trapdoor mangrove_trapdoor oak_trapdoor oxidized_copper_trapdoor pale_oak_trapdoor poplar_trapdoor spruce_trapdoor warped_trapdoor waxed_copper_trapdoor waxed_exposed_copper_trapdoor waxed_oxidized_copper_trapdoor waxed_weathered_copper_trapdoor weathered_copper_trapdoor",
    ),
    (
        "minecraft:triggers_ambient_desert_dry_vegetation_block_sounds",
        "black_terracotta blue_terracotta brown_terracotta cyan_terracotta gray_terracotta green_terracotta light_blue_terracotta light_gray_terracotta lime_terracotta magenta_terracotta orange_terracotta pink_terracotta purple_terracotta red_sand red_terracotta sand terracotta white_terracotta yellow_terracotta",
    ),
    (
        "minecraft:triggers_ambient_desert_sand_block_sounds",
        "red_sand sand",
    ),
    (
        "minecraft:triggers_ambient_dried_ghast_block_sounds",
        "soul_sand soul_soil",
    ),
    (
        "minecraft:turns_into_dirt_path",
        "coarse_dirt dirt grass_block mycelium podzol rooted_dirt",
    ),
    ("minecraft:turns_into_farmland", "dirt dirt_path grass_block"),
    ("minecraft:uncarvable", "bedrock"),
    (
        "minecraft:underwater_bonemeals",
        "brain_coral brain_coral_fan brain_coral_wall_fan bubble_coral bubble_coral_fan bubble_coral_wall_fan fire_coral fire_coral_fan fire_coral_wall_fan horn_coral horn_coral_fan horn_coral_wall_fan seagrass tube_coral tube_coral_fan tube_coral_wall_fan",
    ),
    (
        "minecraft:unstable_bottom_center",
        "acacia_fence_gate bamboo_fence_gate birch_fence_gate cherry_fence_gate crimson_fence_gate dark_oak_fence_gate jungle_fence_gate mangrove_fence_gate oak_fence_gate pale_oak_fence_gate poplar_fence_gate spruce_fence_gate warped_fence_gate",
    ),
    ("minecraft:valid_spawn", "grass_block podzol"),
    ("minecraft:vibration_resonators", "amethyst_block"),
    (
        "minecraft:villager_babies_can_jump_on_bed",
        "black_bed blue_bed brown_bed cyan_bed gray_bed green_bed light_blue_bed light_gray_bed lime_bed magenta_bed orange_bed pink_bed purple_bed red_bed white_bed yellow_bed",
    ),
    (
        "minecraft:villagers_can_sleep_on_bed",
        "black_bed blue_bed brown_bed cyan_bed gray_bed green_bed light_blue_bed light_gray_bed lime_bed magenta_bed orange_bed pink_bed purple_bed red_bed white_bed yellow_bed",
    ),
    (
        "minecraft:wall_corals",
        "brain_coral_wall_fan bubble_coral_wall_fan fire_coral_wall_fan horn_coral_wall_fan tube_coral_wall_fan",
    ),
    (
        "minecraft:wall_hanging_signs",
        "acacia_wall_hanging_sign bamboo_wall_hanging_sign birch_wall_hanging_sign cherry_wall_hanging_sign crimson_wall_hanging_sign dark_oak_wall_hanging_sign jungle_wall_hanging_sign mangrove_wall_hanging_sign oak_wall_hanging_sign pale_oak_wall_hanging_sign poplar_wall_hanging_sign spruce_wall_hanging_sign warped_wall_hanging_sign",
    ),
    (
        "minecraft:wall_post_override",
        "acacia_pressure_plate acacia_sign acacia_wall_sign bamboo_pressure_plate bamboo_sign bamboo_wall_sign birch_pressure_plate birch_sign birch_wall_sign black_banner black_wall_banner blue_banner blue_wall_banner brown_banner brown_wall_banner cactus_flower cherry_pressure_plate cherry_sign cherry_wall_sign copper_torch crimson_pressure_plate crimson_sign crimson_wall_sign cyan_banner cyan_wall_banner dark_oak_pressure_plate dark_oak_sign dark_oak_wall_sign gray_banner gray_wall_banner green_banner green_wall_banner heavy_weighted_pressure_plate jungle_pressure_plate jungle_sign jungle_wall_sign light_blue_banner light_blue_wall_banner light_gray_banner light_gray_wall_banner light_weighted_pressure_plate lime_banner lime_wall_banner magenta_banner magenta_wall_banner mangrove_pressure_plate mangrove_sign mangrove_wall_sign oak_pressure_plate oak_sign oak_wall_sign orange_banner orange_wall_banner pale_oak_pressure_plate pale_oak_sign pale_oak_wall_sign pink_banner pink_wall_banner polished_blackstone_pressure_plate poplar_pressure_plate poplar_sign poplar_wall_sign purple_banner purple_wall_banner red_banner red_wall_banner redstone_torch soul_torch spruce_pressure_plate spruce_sign spruce_wall_sign stone_pressure_plate torch tripwire warped_pressure_plate warped_sign warped_wall_sign white_banner white_wall_banner yellow_banner yellow_wall_banner",
    ),
    (
        "minecraft:wall_signs",
        "acacia_wall_sign bamboo_wall_sign birch_wall_sign cherry_wall_sign crimson_wall_sign dark_oak_wall_sign jungle_wall_sign mangrove_wall_sign oak_wall_sign pale_oak_wall_sign poplar_wall_sign spruce_wall_sign warped_wall_sign",
    ),
    (
        "minecraft:walls",
        "andesite_wall blackstone_wall brick_wall cinnabar_brick_wall cinnabar_wall cobbled_deepslate_wall cobblestone_wall deepslate_brick_wall deepslate_tile_wall diorite_wall end_stone_brick_wall granite_wall mossy_cobblestone_wall mossy_stone_brick_wall mud_brick_wall nether_brick_wall polished_blackstone_brick_wall polished_blackstone_wall polished_cinnabar_wall polished_deepslate_wall polished_sulfur_wall polished_tuff_wall prismarine_wall red_nether_brick_wall red_sandstone_wall resin_brick_wall sandstone_wall stone_brick_wall sulfur_brick_wall sulfur_wall tuff_brick_wall tuff_wall",
    ),
    (
        "minecraft:warped_stems",
        "stripped_warped_hyphae stripped_warped_stem warped_hyphae warped_stem",
    ),
    ("minecraft:wart_blocks", "nether_wart_block warped_wart_block"),
    (
        "minecraft:washed_away_by_fluids",
        "acacia_button acacia_sapling activator_rail air allium attached_melon_stem attached_pumpkin_stem azalea azure_bluet bamboo_button bamboo_sapling beetroots big_dripleaf big_dripleaf_stem birch_button birch_sapling black_candle black_carpet blue_candle blue_carpet blue_orchid brain_coral brain_coral_fan brain_coral_wall_fan brown_candle brown_carpet brown_mushroom bubble_coral bubble_coral_fan bubble_coral_wall_fan bush cactus_flower candle carrots cave_air cave_vines cave_vines_plant cherry_button cherry_sapling chorus_flower chorus_plant closed_eyeblossom cobweb cocoa comparator copper_golem_statue copper_torch copper_wall_torch cornflower creeper_head creeper_wall_head crimson_button crimson_fungus crimson_roots cyan_candle cyan_carpet dandelion dark_oak_button dark_oak_sapling dead_bush detector_rail dragon_head dragon_wall_head end_rod exposed_copper_golem_statue fern fire fire_coral fire_coral_fan fire_coral_wall_fan firefly_bush flower_pot flowering_azalea frogspawn glow_lichen golden_dandelion gray_candle gray_carpet green_candle green_carpet hanging_roots heavy_core horn_coral horn_coral_fan horn_coral_wall_fan jungle_button jungle_sapling kelp kelp_plant large_fern lava leaf_litter lever light light_blue_candle light_blue_carpet light_gray_candle light_gray_carpet lilac lily_of_the_valley lily_pad lime_candle lime_carpet magenta_candle magenta_carpet mangrove_button mangrove_propagule melon_stem moss_carpet nether_sprouts nether_wart oak_button oak_sapling open_eyeblossom orange_candle orange_carpet orange_tulip oxeye_daisy oxidized_copper_golem_statue pale_hanging_moss pale_moss_carpet pale_oak_button pale_oak_sapling peony piglin_head piglin_wall_head pink_candle pink_carpet pink_petals pink_tulip pitcher_crop pitcher_plant player_head player_wall_head polished_blackstone_button poplar_button poplar_sapling poppy potatoes potted_acacia_sapling potted_allium potted_azalea_bush potted_azure_bluet potted_bamboo potted_birch_sapling potted_blue_orchid potted_brown_mushroom potted_cactus potted_cherry_sapling potted_closed_eyeblossom potted_cornflower potted_crimson_fungus potted_crimson_roots potted_dandelion potted_dark_oak_sapling potted_dead_bush potted_fern potted_flowering_azalea_bush potted_golden_dandelion potted_jungle_sapling potted_lily_of_the_valley potted_mangrove_propagule potted_oak_sapling potted_open_eyeblossom potted_orange_tulip potted_oxeye_daisy potted_pale_oak_sapling potted_pink_tulip potted_poplar_sapling potted_poppy potted_red_mushroom potted_red_tulip potted_spruce_sapling potted_torchflower potted_warped_fungus potted_warped_roots potted_white_tulip potted_wither_rose powder_snow powered_rail pumpkin_stem purple_candle purple_carpet rail red_candle red_carpet red_mushroom red_shrub red_tulip redstone_torch redstone_wall_torch redstone_wire repeater resin_clump rose_bush scaffolding sea_pickle seagrass shelf_mushroom short_dry_grass short_grass skeleton_skull skeleton_wall_skull small_dripleaf snow soul_fire soul_torch soul_wall_torch spore_blossom spruce_button spruce_sapling stone_button straw_bed sunflower sweet_berry_bush tall_dry_grass tall_grass tall_seagrass torch torchflower torchflower_crop tripwire tripwire_hook tube_coral tube_coral_fan tube_coral_wall_fan twisting_vines twisting_vines_plant vine void_air wall_torch warped_button warped_fungus warped_roots water waxed_copper_golem_statue waxed_exposed_copper_golem_statue waxed_oxidized_copper_golem_statue waxed_weathered_copper_golem_statue weathered_copper_golem_statue weeping_vines weeping_vines_plant wheat white_candle white_carpet white_tulip wildflowers wither_rose wither_skeleton_skull wither_skeleton_wall_skull yellow_candle yellow_carpet zombie_head zombie_wall_head",
    ),
    (
        "minecraft:wither_immune",
        "barrier bedrock chain_command_block command_block end_gateway end_portal end_portal_frame jigsaw light moving_piston reinforced_deepslate repeating_command_block structure_block test_block test_instance_block",
    ),
    ("minecraft:wither_immune_to", "wither_rose"),
    ("minecraft:wither_skeleton_immune_to", "wither_rose"),
    ("minecraft:wither_summon_base_blocks", "soul_sand soul_soil"),
    (
        "minecraft:wolves_spawnable_on",
        "coarse_dirt grass_block podzol snow snow_block",
    ),
    (
        "minecraft:wooden_buttons",
        "acacia_button bamboo_button birch_button cherry_button crimson_button dark_oak_button jungle_button mangrove_button oak_button pale_oak_button poplar_button spruce_button warped_button",
    ),
    (
        "minecraft:wooden_doors",
        "acacia_door bamboo_door birch_door cherry_door crimson_door dark_oak_door jungle_door mangrove_door oak_door pale_oak_door poplar_door spruce_door warped_door",
    ),
    (
        "minecraft:wooden_fences",
        "acacia_fence bamboo_fence birch_fence cherry_fence crimson_fence dark_oak_fence jungle_fence mangrove_fence oak_fence pale_oak_fence poplar_fence spruce_fence warped_fence",
    ),
    (
        "minecraft:wooden_pressure_plates",
        "acacia_pressure_plate bamboo_pressure_plate birch_pressure_plate cherry_pressure_plate crimson_pressure_plate dark_oak_pressure_plate jungle_pressure_plate mangrove_pressure_plate oak_pressure_plate pale_oak_pressure_plate poplar_pressure_plate spruce_pressure_plate warped_pressure_plate",
    ),
    (
        "minecraft:wooden_shelves",
        "acacia_shelf bamboo_shelf birch_shelf cherry_shelf crimson_shelf dark_oak_shelf jungle_shelf mangrove_shelf oak_shelf pale_oak_shelf poplar_shelf spruce_shelf warped_shelf",
    ),
    (
        "minecraft:wooden_slabs",
        "acacia_slab bamboo_slab birch_slab cherry_slab crimson_slab dark_oak_slab jungle_slab mangrove_slab oak_slab pale_oak_slab spruce_slab warped_slab",
    ),
    (
        "minecraft:wooden_stairs",
        "acacia_stairs bamboo_stairs birch_stairs cherry_stairs crimson_stairs dark_oak_stairs jungle_stairs mangrove_stairs oak_stairs pale_oak_stairs poplar_stairs spruce_stairs warped_stairs",
    ),
    (
        "minecraft:wooden_trapdoors",
        "acacia_trapdoor bamboo_trapdoor birch_trapdoor cherry_trapdoor crimson_trapdoor dark_oak_trapdoor jungle_trapdoor mangrove_trapdoor oak_trapdoor pale_oak_trapdoor poplar_trapdoor spruce_trapdoor warped_trapdoor",
    ),
    (
        "minecraft:wool",
        "black_wool blue_wool brown_wool cyan_wool gray_wool green_wool light_blue_wool light_gray_wool lime_wool magenta_wool orange_wool pink_wool purple_wool red_wool white_wool yellow_wool",
    ),
    (
        "minecraft:wool_carpets",
        "black_carpet blue_carpet brown_carpet cyan_carpet gray_carpet green_carpet light_blue_carpet light_gray_carpet lime_carpet magenta_carpet orange_carpet pink_carpet purple_carpet red_carpet white_carpet yellow_carpet",
    ),
    (
        "minecraft:wool_slabs",
        "black_wool_slab blue_wool_slab brown_wool_slab cyan_wool_slab gray_wool_slab green_wool_slab light_blue_wool_slab light_gray_wool_slab lime_wool_slab magenta_wool_slab orange_wool_slab pink_wool_slab purple_wool_slab red_wool_slab white_wool_slab yellow_wool_slab",
    ),
    (
        "minecraft:wool_stairs",
        "black_wool_stairs blue_wool_stairs brown_wool_stairs cyan_wool_stairs gray_wool_stairs green_wool_stairs light_blue_wool_stairs light_gray_wool_stairs lime_wool_stairs magenta_wool_stairs orange_wool_stairs pink_wool_stairs purple_wool_stairs red_wool_stairs white_wool_stairs yellow_wool_stairs",
    ),
];

pub static ITEM_TAGS: &[(&str, &str)] = &[
    (
        "c:animal_foods",
        "allium apple azure_bluet bamboo beef beetroot beetroot_seeds blue_orchid cactus cactus_flower carrot cherry_leaves chicken chorus_flower cod cooked_beef cooked_chicken cooked_cod cooked_mutton cooked_porkchop cooked_rabbit cooked_salmon cornflower crimson_fungus dandelion enchanted_golden_apple flowering_azalea flowering_azalea_leaves glow_berries golden_apple golden_carrot hay_block lilac lily_of_the_valley mangrove_propagule melon_seeds mutton open_eyeblossom orange_tulip oxeye_daisy peony pink_petals pink_tulip pitcher_plant pitcher_pod poppy porkchop potato pufferfish pumpkin_seeds rabbit rabbit_stew red_tulip rose_bush rotten_flesh salmon seagrass slime_ball spider_eye spore_blossom sugar sunflower sweet_berries torchflower torchflower_seeds tropical_fish tropical_fish_bucket warped_fungus wheat wheat_seeds white_tulip wildflowers wither_rose",
    ),
    (
        "c:armors",
        "chainmail_boots chainmail_chestplate chainmail_helmet chainmail_leggings copper_boots copper_chestplate copper_helmet copper_horse_armor copper_leggings copper_nautilus_armor diamond_boots diamond_chestplate diamond_helmet diamond_horse_armor diamond_leggings diamond_nautilus_armor golden_boots golden_chestplate golden_helmet golden_horse_armor golden_leggings golden_nautilus_armor iron_boots iron_chestplate iron_helmet iron_horse_armor iron_leggings iron_nautilus_armor leather_boots leather_chestplate leather_helmet leather_horse_armor leather_leggings netherite_boots netherite_chestplate netherite_helmet netherite_horse_armor netherite_leggings netherite_nautilus_armor turtle_helmet wolf_armor",
    ),
    (
        "c:armors/horse",
        "copper_horse_armor diamond_horse_armor golden_horse_armor iron_horse_armor leather_horse_armor netherite_horse_armor",
    ),
    (
        "c:armors/humanoid",
        "chainmail_boots chainmail_chestplate chainmail_helmet chainmail_leggings copper_boots copper_chestplate copper_helmet copper_leggings diamond_boots diamond_chestplate diamond_helmet diamond_leggings golden_boots golden_chestplate golden_helmet golden_leggings iron_boots iron_chestplate iron_helmet iron_leggings leather_boots leather_chestplate leather_helmet leather_leggings netherite_boots netherite_chestplate netherite_helmet netherite_leggings turtle_helmet",
    ),
    (
        "c:armors/nautilus",
        "copper_nautilus_armor diamond_nautilus_armor golden_nautilus_armor iron_nautilus_armor netherite_nautilus_armor",
    ),
    ("c:armors/wolf", "wolf_armor"),
    ("c:barrels", "barrel"),
    ("c:barrels/wooden", "barrel"),
    (
        "c:bars",
        "copper_bars exposed_copper_bars iron_bars oxidized_copper_bars waxed_copper_bars waxed_exposed_copper_bars waxed_oxidized_copper_bars waxed_weathered_copper_bars weathered_copper_bars",
    ),
    (
        "c:bars/copper",
        "copper_bars exposed_copper_bars oxidized_copper_bars waxed_copper_bars waxed_exposed_copper_bars waxed_oxidized_copper_bars waxed_weathered_copper_bars weathered_copper_bars",
    ),
    ("c:bars/iron", "iron_bars"),
    ("c:bones", "bone"),
    ("c:bookshelves", "bookshelf"),
    ("c:bricks", "brick nether_brick resin_brick"),
    ("c:bricks/nether", "nether_brick"),
    ("c:bricks/normal", "brick"),
    ("c:bricks/resin", "resin_brick"),
    (
        "c:buckets",
        "axolotl_bucket bucket cod_bucket lava_bucket milk_bucket powder_snow_bucket pufferfish_bucket salmon_bucket sulfur_cube_bucket tadpole_bucket tropical_fish_bucket water_bucket",
    ),
    ("c:buckets/empty", "bucket"),
    ("c:buckets/entity_dry", "sulfur_cube_bucket"),
    (
        "c:buckets/entity_water",
        "axolotl_bucket cod_bucket pufferfish_bucket salmon_bucket tadpole_bucket tropical_fish_bucket",
    ),
    ("c:buckets/lava", "lava_bucket"),
    ("c:buckets/milk", "milk_bucket"),
    ("c:buckets/powder_snow", "powder_snow_bucket"),
    ("c:buckets/water", "water_bucket"),
    ("c:budding_blocks", "budding_amethyst"),
    (
        "c:buds",
        "large_amethyst_bud medium_amethyst_bud small_amethyst_bud",
    ),
    (
        "c:chains",
        "copper_chain exposed_copper_chain iron_chain oxidized_copper_chain waxed_copper_chain waxed_exposed_copper_chain waxed_oxidized_copper_chain waxed_weathered_copper_chain weathered_copper_chain",
    ),
    (
        "c:chests",
        "chest copper_chest ender_chest exposed_copper_chest oxidized_copper_chest trapped_chest waxed_copper_chest waxed_exposed_copper_chest waxed_oxidized_copper_chest waxed_weathered_copper_chest weathered_copper_chest",
    ),
    ("c:chests/ender", "ender_chest"),
    ("c:chests/trapped", "trapped_chest"),
    ("c:chests/wooden", "chest trapped_chest"),
    ("c:clumps", "resin_clump"),
    ("c:clumps/resin", "resin_clump"),
    ("c:clusters", "amethyst_cluster"),
    (
        "c:cobblestones",
        "cobbled_deepslate cobblestone infested_cobblestone mossy_cobblestone",
    ),
    ("c:cobblestones/deepslate", "cobbled_deepslate"),
    ("c:cobblestones/infested", "infested_cobblestone"),
    ("c:cobblestones/mossy", "mossy_cobblestone"),
    ("c:cobblestones/normal", "cobblestone"),
    (
        "c:concrete_powders",
        "black_concrete_powder blue_concrete_powder brown_concrete_powder cyan_concrete_powder gray_concrete_powder green_concrete_powder light_blue_concrete_powder light_gray_concrete_powder lime_concrete_powder magenta_concrete_powder orange_concrete_powder pink_concrete_powder purple_concrete_powder red_concrete_powder white_concrete_powder yellow_concrete_powder",
    ),
    (
        "c:concretes",
        "black_concrete blue_concrete brown_concrete cyan_concrete gray_concrete green_concrete light_blue_concrete light_gray_concrete lime_concrete magenta_concrete orange_concrete pink_concrete purple_concrete red_concrete white_concrete yellow_concrete",
    ),
    (
        "c:crops",
        "beetroot cactus carrot cocoa_beans melon nether_wart potato pumpkin sugar_cane wheat",
    ),
    ("c:crops/beetroot", "beetroot"),
    ("c:crops/cactus", "cactus"),
    ("c:crops/carrot", "carrot"),
    ("c:crops/cocoa_bean", "cocoa_beans"),
    ("c:crops/melon", "melon"),
    ("c:crops/nether_wart", "nether_wart"),
    ("c:crops/potato", "potato"),
    ("c:crops/pumpkin", "pumpkin"),
    ("c:crops/sugar_cane", "sugar_cane"),
    ("c:crops/wheat", "wheat"),
    ("c:drink_containing/bottle", "honey_bottle ominous_bottle potion"),
    ("c:drink_containing/bucket", "milk_bucket"),
    ("c:drinks", "honey_bottle milk_bucket ominous_bottle potion"),
    ("c:drinks/honey", "honey_bottle"),
    ("c:drinks/juice", ""),
    ("c:drinks/magic", "ominous_bottle potion"),
    ("c:drinks/milk", "milk_bucket"),
    ("c:drinks/ominous", "ominous_bottle"),
    ("c:drinks/water", ""),
    ("c:drinks/watery", "potion"),
    ("c:dusts", "glowstone_dust redstone"),
    ("c:dusts/glowstone", "glowstone_dust"),
    ("c:dusts/redstone", "redstone"),
    (
        "c:dyeable",
        "black_bed black_bundle black_carpet black_cushion black_harness black_shulker_box black_wool black_wool_slab black_wool_stairs blue_bed blue_bundle blue_carpet blue_cushion blue_harness blue_shulker_box blue_wool blue_wool_slab blue_wool_stairs brown_bed brown_bundle brown_carpet brown_cushion brown_harness brown_shulker_box brown_wool brown_wool_slab brown_wool_stairs bundle candle cyan_bed cyan_bundle cyan_carpet cyan_cushion cyan_harness cyan_shulker_box cyan_wool cyan_wool_slab cyan_wool_stairs firework_star glass glass_pane gray_bed gray_bundle gray_carpet gray_cushion gray_harness gray_shulker_box gray_wool gray_wool_slab gray_wool_stairs green_bed green_bundle green_carpet green_cushion green_harness green_shulker_box green_wool green_wool_slab green_wool_stairs leather_boots leather_chestplate leather_helmet leather_horse_armor leather_leggings light_blue_bed light_blue_bundle light_blue_carpet light_blue_cushion light_blue_harness light_blue_shulker_box light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_bed light_gray_bundle light_gray_carpet light_gray_cushion light_gray_harness light_gray_shulker_box light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_bed lime_bundle lime_carpet lime_cushion lime_harness lime_shulker_box lime_wool lime_wool_slab lime_wool_stairs magenta_bed magenta_bundle magenta_carpet magenta_cushion magenta_harness magenta_shulker_box magenta_wool magenta_wool_slab magenta_wool_stairs orange_bed orange_bundle orange_carpet orange_cushion orange_harness orange_shulker_box orange_wool orange_wool_slab orange_wool_stairs pink_bed pink_bundle pink_carpet pink_cushion pink_harness pink_shulker_box pink_wool pink_wool_slab pink_wool_stairs purple_bed purple_bundle purple_carpet purple_cushion purple_harness purple_shulker_box purple_wool purple_wool_slab purple_wool_stairs red_bed red_bundle red_carpet red_cushion red_harness red_shulker_box red_wool red_wool_slab red_wool_stairs shulker_box terracotta white_bed white_bundle white_carpet white_cushion white_harness white_shulker_box white_wool white_wool_slab white_wool_stairs wolf_armor yellow_bed yellow_bundle yellow_carpet yellow_cushion yellow_harness yellow_shulker_box yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "c:dyeable/dynamic",
        "firework_star leather_boots leather_chestplate leather_helmet leather_horse_armor leather_leggings wolf_armor",
    ),
    (
        "c:dyeable/simple",
        "black_bed black_bundle black_carpet black_cushion black_harness black_shulker_box black_wool black_wool_slab black_wool_stairs blue_bed blue_bundle blue_carpet blue_cushion blue_harness blue_shulker_box blue_wool blue_wool_slab blue_wool_stairs brown_bed brown_bundle brown_carpet brown_cushion brown_harness brown_shulker_box brown_wool brown_wool_slab brown_wool_stairs bundle candle cyan_bed cyan_bundle cyan_carpet cyan_cushion cyan_harness cyan_shulker_box cyan_wool cyan_wool_slab cyan_wool_stairs glass glass_pane gray_bed gray_bundle gray_carpet gray_cushion gray_harness gray_shulker_box gray_wool gray_wool_slab gray_wool_stairs green_bed green_bundle green_carpet green_cushion green_harness green_shulker_box green_wool green_wool_slab green_wool_stairs light_blue_bed light_blue_bundle light_blue_carpet light_blue_cushion light_blue_harness light_blue_shulker_box light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_bed light_gray_bundle light_gray_carpet light_gray_cushion light_gray_harness light_gray_shulker_box light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_bed lime_bundle lime_carpet lime_cushion lime_harness lime_shulker_box lime_wool lime_wool_slab lime_wool_stairs magenta_bed magenta_bundle magenta_carpet magenta_cushion magenta_harness magenta_shulker_box magenta_wool magenta_wool_slab magenta_wool_stairs orange_bed orange_bundle orange_carpet orange_cushion orange_harness orange_shulker_box orange_wool orange_wool_slab orange_wool_stairs pink_bed pink_bundle pink_carpet pink_cushion pink_harness pink_shulker_box pink_wool pink_wool_slab pink_wool_stairs purple_bed purple_bundle purple_carpet purple_cushion purple_harness purple_shulker_box purple_wool purple_wool_slab purple_wool_stairs red_bed red_bundle red_carpet red_cushion red_harness red_shulker_box red_wool red_wool_slab red_wool_stairs shulker_box terracotta white_bed white_bundle white_carpet white_cushion white_harness white_shulker_box white_wool white_wool_slab white_wool_stairs yellow_bed yellow_bundle yellow_carpet yellow_cushion yellow_harness yellow_shulker_box yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "c:dyeable/simple/redyeable",
        "black_bed black_bundle black_carpet black_cushion black_harness black_shulker_box black_wool black_wool_slab black_wool_stairs blue_bed blue_bundle blue_carpet blue_cushion blue_harness blue_shulker_box blue_wool blue_wool_slab blue_wool_stairs brown_bed brown_bundle brown_carpet brown_cushion brown_harness brown_shulker_box brown_wool brown_wool_slab brown_wool_stairs cyan_bed cyan_bundle cyan_carpet cyan_cushion cyan_harness cyan_shulker_box cyan_wool cyan_wool_slab cyan_wool_stairs gray_bed gray_bundle gray_carpet gray_cushion gray_harness gray_shulker_box gray_wool gray_wool_slab gray_wool_stairs green_bed green_bundle green_carpet green_cushion green_harness green_shulker_box green_wool green_wool_slab green_wool_stairs light_blue_bed light_blue_bundle light_blue_carpet light_blue_cushion light_blue_harness light_blue_shulker_box light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_bed light_gray_bundle light_gray_carpet light_gray_cushion light_gray_harness light_gray_shulker_box light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_bed lime_bundle lime_carpet lime_cushion lime_harness lime_shulker_box lime_wool lime_wool_slab lime_wool_stairs magenta_bed magenta_bundle magenta_carpet magenta_cushion magenta_harness magenta_shulker_box magenta_wool magenta_wool_slab magenta_wool_stairs orange_bed orange_bundle orange_carpet orange_cushion orange_harness orange_shulker_box orange_wool orange_wool_slab orange_wool_stairs pink_bed pink_bundle pink_carpet pink_cushion pink_harness pink_shulker_box pink_wool pink_wool_slab pink_wool_stairs purple_bed purple_bundle purple_carpet purple_cushion purple_harness purple_shulker_box purple_wool purple_wool_slab purple_wool_stairs red_bed red_bundle red_carpet red_cushion red_harness red_shulker_box red_wool red_wool_slab red_wool_stairs white_bed white_bundle white_carpet white_cushion white_harness white_shulker_box white_wool white_wool_slab white_wool_stairs yellow_bed yellow_bundle yellow_carpet yellow_cushion yellow_harness yellow_shulker_box yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "c:dyeable/simple/undyed",
        "bundle candle glass glass_pane shulker_box terracotta",
    ),
    (
        "c:dyed",
        "black_banner black_bed black_bundle black_candle black_carpet black_concrete black_concrete_powder black_cushion black_glazed_terracotta black_harness black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wool black_wool_slab black_wool_stairs blue_banner blue_bed blue_bundle blue_candle blue_carpet blue_concrete blue_concrete_powder blue_cushion blue_glazed_terracotta blue_harness blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wool blue_wool_slab blue_wool_stairs brown_banner brown_bed brown_bundle brown_candle brown_carpet brown_concrete brown_concrete_powder brown_cushion brown_glazed_terracotta brown_harness brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wool brown_wool_slab brown_wool_stairs cyan_banner cyan_bed cyan_bundle cyan_candle cyan_carpet cyan_concrete cyan_concrete_powder cyan_cushion cyan_glazed_terracotta cyan_harness cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wool cyan_wool_slab cyan_wool_stairs gray_banner gray_bed gray_bundle gray_candle gray_carpet gray_concrete gray_concrete_powder gray_cushion gray_glazed_terracotta gray_harness gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wool gray_wool_slab gray_wool_stairs green_banner green_bed green_bundle green_candle green_carpet green_concrete green_concrete_powder green_cushion green_glazed_terracotta green_harness green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wool green_wool_slab green_wool_stairs light_blue_banner light_blue_bed light_blue_bundle light_blue_candle light_blue_carpet light_blue_concrete light_blue_concrete_powder light_blue_cushion light_blue_glazed_terracotta light_blue_harness light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_banner light_gray_bed light_gray_bundle light_gray_candle light_gray_carpet light_gray_concrete light_gray_concrete_powder light_gray_cushion light_gray_glazed_terracotta light_gray_harness light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_banner lime_bed lime_bundle lime_candle lime_carpet lime_concrete lime_concrete_powder lime_cushion lime_glazed_terracotta lime_harness lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wool lime_wool_slab lime_wool_stairs magenta_banner magenta_bed magenta_bundle magenta_candle magenta_carpet magenta_concrete magenta_concrete_powder magenta_cushion magenta_glazed_terracotta magenta_harness magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wool magenta_wool_slab magenta_wool_stairs orange_banner orange_bed orange_bundle orange_candle orange_carpet orange_concrete orange_concrete_powder orange_cushion orange_glazed_terracotta orange_harness orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wool orange_wool_slab orange_wool_stairs pink_banner pink_bed pink_bundle pink_candle pink_carpet pink_concrete pink_concrete_powder pink_cushion pink_glazed_terracotta pink_harness pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wool pink_wool_slab pink_wool_stairs purple_banner purple_bed purple_bundle purple_candle purple_carpet purple_concrete purple_concrete_powder purple_cushion purple_glazed_terracotta purple_harness purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wool purple_wool_slab purple_wool_stairs red_banner red_bed red_bundle red_candle red_carpet red_concrete red_concrete_powder red_cushion red_glazed_terracotta red_harness red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wool red_wool_slab red_wool_stairs white_banner white_bed white_bundle white_candle white_carpet white_concrete white_concrete_powder white_cushion white_glazed_terracotta white_harness white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wool white_wool_slab white_wool_stairs yellow_banner yellow_bed yellow_bundle yellow_candle yellow_carpet yellow_concrete yellow_concrete_powder yellow_cushion yellow_glazed_terracotta yellow_harness yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "c:dyed/black",
        "black_banner black_bed black_bundle black_candle black_carpet black_concrete black_concrete_powder black_cushion black_glazed_terracotta black_harness black_shulker_box black_stained_glass black_stained_glass_pane black_terracotta black_wool black_wool_slab black_wool_stairs",
    ),
    (
        "c:dyed/blue",
        "blue_banner blue_bed blue_bundle blue_candle blue_carpet blue_concrete blue_concrete_powder blue_cushion blue_glazed_terracotta blue_harness blue_shulker_box blue_stained_glass blue_stained_glass_pane blue_terracotta blue_wool blue_wool_slab blue_wool_stairs",
    ),
    (
        "c:dyed/brown",
        "brown_banner brown_bed brown_bundle brown_candle brown_carpet brown_concrete brown_concrete_powder brown_cushion brown_glazed_terracotta brown_harness brown_shulker_box brown_stained_glass brown_stained_glass_pane brown_terracotta brown_wool brown_wool_slab brown_wool_stairs",
    ),
    (
        "c:dyed/cyan",
        "cyan_banner cyan_bed cyan_bundle cyan_candle cyan_carpet cyan_concrete cyan_concrete_powder cyan_cushion cyan_glazed_terracotta cyan_harness cyan_shulker_box cyan_stained_glass cyan_stained_glass_pane cyan_terracotta cyan_wool cyan_wool_slab cyan_wool_stairs",
    ),
    (
        "c:dyed/gray",
        "gray_banner gray_bed gray_bundle gray_candle gray_carpet gray_concrete gray_concrete_powder gray_cushion gray_glazed_terracotta gray_harness gray_shulker_box gray_stained_glass gray_stained_glass_pane gray_terracotta gray_wool gray_wool_slab gray_wool_stairs",
    ),
    (
        "c:dyed/green",
        "green_banner green_bed green_bundle green_candle green_carpet green_concrete green_concrete_powder green_cushion green_glazed_terracotta green_harness green_shulker_box green_stained_glass green_stained_glass_pane green_terracotta green_wool green_wool_slab green_wool_stairs",
    ),
    (
        "c:dyed/light_blue",
        "light_blue_banner light_blue_bed light_blue_bundle light_blue_candle light_blue_carpet light_blue_concrete light_blue_concrete_powder light_blue_cushion light_blue_glazed_terracotta light_blue_harness light_blue_shulker_box light_blue_stained_glass light_blue_stained_glass_pane light_blue_terracotta light_blue_wool light_blue_wool_slab light_blue_wool_stairs",
    ),
    (
        "c:dyed/light_gray",
        "light_gray_banner light_gray_bed light_gray_bundle light_gray_candle light_gray_carpet light_gray_concrete light_gray_concrete_powder light_gray_cushion light_gray_glazed_terracotta light_gray_harness light_gray_shulker_box light_gray_stained_glass light_gray_stained_glass_pane light_gray_terracotta light_gray_wool light_gray_wool_slab light_gray_wool_stairs",
    ),
    (
        "c:dyed/lime",
        "lime_banner lime_bed lime_bundle lime_candle lime_carpet lime_concrete lime_concrete_powder lime_cushion lime_glazed_terracotta lime_harness lime_shulker_box lime_stained_glass lime_stained_glass_pane lime_terracotta lime_wool lime_wool_slab lime_wool_stairs",
    ),
    (
        "c:dyed/magenta",
        "magenta_banner magenta_bed magenta_bundle magenta_candle magenta_carpet magenta_concrete magenta_concrete_powder magenta_cushion magenta_glazed_terracotta magenta_harness magenta_shulker_box magenta_stained_glass magenta_stained_glass_pane magenta_terracotta magenta_wool magenta_wool_slab magenta_wool_stairs",
    ),
    (
        "c:dyed/orange",
        "orange_banner orange_bed orange_bundle orange_candle orange_carpet orange_concrete orange_concrete_powder orange_cushion orange_glazed_terracotta orange_harness orange_shulker_box orange_stained_glass orange_stained_glass_pane orange_terracotta orange_wool orange_wool_slab orange_wool_stairs",
    ),
    (
        "c:dyed/pink",
        "pink_banner pink_bed pink_bundle pink_candle pink_carpet pink_concrete pink_concrete_powder pink_cushion pink_glazed_terracotta pink_harness pink_shulker_box pink_stained_glass pink_stained_glass_pane pink_terracotta pink_wool pink_wool_slab pink_wool_stairs",
    ),
    (
        "c:dyed/purple",
        "purple_banner purple_bed purple_bundle purple_candle purple_carpet purple_concrete purple_concrete_powder purple_cushion purple_glazed_terracotta purple_harness purple_shulker_box purple_stained_glass purple_stained_glass_pane purple_terracotta purple_wool purple_wool_slab purple_wool_stairs",
    ),
    (
        "c:dyed/red",
        "red_banner red_bed red_bundle red_candle red_carpet red_concrete red_concrete_powder red_cushion red_glazed_terracotta red_harness red_shulker_box red_stained_glass red_stained_glass_pane red_terracotta red_wool red_wool_slab red_wool_stairs",
    ),
    (
        "c:dyed/white",
        "white_banner white_bed white_bundle white_candle white_carpet white_concrete white_concrete_powder white_cushion white_glazed_terracotta white_harness white_shulker_box white_stained_glass white_stained_glass_pane white_terracotta white_wool white_wool_slab white_wool_stairs",
    ),
    (
        "c:dyed/yellow",
        "yellow_banner yellow_bed yellow_bundle yellow_candle yellow_carpet yellow_concrete yellow_concrete_powder yellow_cushion yellow_glazed_terracotta yellow_harness yellow_shulker_box yellow_stained_glass yellow_stained_glass_pane yellow_terracotta yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "c:dyes",
        "black_dye blue_dye brown_dye cyan_dye gray_dye green_dye light_blue_dye light_gray_dye lime_dye magenta_dye orange_dye pink_dye purple_dye red_dye white_dye yellow_dye",
    ),
    ("c:dyes/black", "black_dye"),
    ("c:dyes/blue", "blue_dye"),
    ("c:dyes/brown", "brown_dye"),
    ("c:dyes/cyan", "cyan_dye"),
    ("c:dyes/gray", "gray_dye"),
    ("c:dyes/green", "green_dye"),
    ("c:dyes/light_blue", "light_blue_dye"),
    ("c:dyes/light_gray", "light_gray_dye"),
    ("c:dyes/lime", "lime_dye"),
    ("c:dyes/magenta", "magenta_dye"),
    ("c:dyes/orange", "orange_dye"),
    ("c:dyes/pink", "pink_dye"),
    ("c:dyes/purple", "purple_dye"),
    ("c:dyes/red", "red_dye"),
    ("c:dyes/white", "white_dye"),
    ("c:dyes/yellow", "yellow_dye"),
    ("c:eggs", "blue_egg brown_egg egg"),
    (
        "c:enchantables",
        "bow brush carrot_on_a_stick carved_pumpkin chainmail_boots chainmail_chestplate chainmail_helmet chainmail_leggings compass copper_axe copper_boots copper_chestplate copper_helmet copper_hoe copper_leggings copper_pickaxe copper_shovel copper_spear copper_sword creeper_head crossbow diamond_axe diamond_boots diamond_chestplate diamond_helmet diamond_hoe diamond_leggings diamond_pickaxe diamond_shovel diamond_spear diamond_sword dragon_head elytra fishing_rod flint_and_steel golden_axe golden_boots golden_chestplate golden_helmet golden_hoe golden_leggings golden_pickaxe golden_shovel golden_spear golden_sword iron_axe iron_boots iron_chestplate iron_helmet iron_hoe iron_leggings iron_pickaxe iron_shovel iron_spear iron_sword leather_boots leather_chestplate leather_helmet leather_leggings mace netherite_axe netherite_boots netherite_chestplate netherite_helmet netherite_hoe netherite_leggings netherite_pickaxe netherite_shovel netherite_spear netherite_sword piglin_head player_head shears shield skeleton_skull stone_axe stone_hoe stone_pickaxe stone_shovel stone_spear stone_sword trident turtle_helmet warped_fungus_on_a_stick wither_skeleton_skull wooden_axe wooden_hoe wooden_pickaxe wooden_shovel wooden_spear wooden_sword zombie_head",
    ),
    ("c:end_stones", "end_stone"),
    ("c:ender_pearls", "ender_pearl"),
    ("c:feathers", "feather"),
    (
        "c:fence_gates",
        "acacia_fence_gate bamboo_fence_gate birch_fence_gate cherry_fence_gate crimson_fence_gate dark_oak_fence_gate jungle_fence_gate mangrove_fence_gate oak_fence_gate pale_oak_fence_gate spruce_fence_gate warped_fence_gate",
    ),
    (
        "c:fence_gates/wooden",
        "acacia_fence_gate bamboo_fence_gate birch_fence_gate cherry_fence_gate crimson_fence_gate dark_oak_fence_gate jungle_fence_gate mangrove_fence_gate oak_fence_gate pale_oak_fence_gate spruce_fence_gate warped_fence_gate",
    ),
    (
        "c:fences",
        "acacia_fence bamboo_fence birch_fence cherry_fence crimson_fence dark_oak_fence jungle_fence mangrove_fence nether_brick_fence oak_fence pale_oak_fence spruce_fence warped_fence",
    ),
    ("c:fences/nether_brick", "nether_brick_fence"),
    (
        "c:fences/wooden",
        "acacia_fence bamboo_fence birch_fence cherry_fence crimson_fence dark_oak_fence jungle_fence mangrove_fence oak_fence pale_oak_fence spruce_fence warped_fence",
    ),
    ("c:fertilizers", "bone_meal"),
    (
        "c:flowers",
        "allium azure_bluet blue_orchid cactus_flower cherry_leaves chorus_flower closed_eyeblossom cornflower dandelion flowering_azalea flowering_azalea_leaves lilac lily_of_the_valley mangrove_propagule open_eyeblossom orange_tulip oxeye_daisy peony pink_petals pink_tulip pitcher_plant poppy red_tulip rose_bush spore_blossom sunflower torchflower white_tulip wildflowers wither_rose",
    ),
    (
        "c:flowers/small",
        "allium azure_bluet blue_orchid closed_eyeblossom cornflower dandelion lily_of_the_valley open_eyeblossom orange_tulip oxeye_daisy pink_tulip poppy red_tulip torchflower white_tulip wither_rose",
    ),
    ("c:flowers/tall", "lilac peony pitcher_plant rose_bush sunflower"),
    (
        "c:foods",
        "apple baked_potato beef beetroot beetroot_soup bread cake carrot chicken chorus_fruit cod cooked_beef cooked_chicken cooked_cod cooked_mutton cooked_porkchop cooked_rabbit cooked_salmon cookie dried_kelp enchanted_golden_apple glow_berries golden_apple golden_carrot honey_bottle melon_slice mushroom_stew mutton ominous_bottle poisonous_potato porkchop potato pufferfish pumpkin_pie rabbit rabbit_stew rotten_flesh salmon spider_eye suspicious_stew sweet_berries tropical_fish",
    ),
    ("c:foods/berry", "glow_berries sweet_berries"),
    ("c:foods/bread", "bread"),
    ("c:foods/candy", ""),
    ("c:foods/cooked_fish", "cooked_cod cooked_salmon"),
    (
        "c:foods/cooked_meat",
        "cooked_beef cooked_chicken cooked_mutton cooked_porkchop cooked_rabbit",
    ),
    ("c:foods/cookie", "cookie"),
    ("c:foods/dough", ""),
    ("c:foods/edible_when_placed", "cake"),
    (
        "c:foods/food_poisoning",
        "chicken poisonous_potato pufferfish rotten_flesh spider_eye",
    ),
    (
        "c:foods/fruit",
        "apple chorus_fruit enchanted_golden_apple golden_apple melon_slice",
    ),
    (
        "c:foods/golden",
        "enchanted_golden_apple golden_apple golden_carrot",
    ),
    ("c:foods/pie", "pumpkin_pie"),
    ("c:foods/raw_fish", "cod pufferfish salmon tropical_fish"),
    ("c:foods/raw_meat", "beef chicken mutton porkchop rabbit"),
    (
        "c:foods/soup",
        "beetroot_soup mushroom_stew rabbit_stew suspicious_stew",
    ),
    ("c:foods/vegetable", "beetroot carrot golden_carrot potato"),
    (
        "c:froglights",
        "ochre_froglight pearlescent_froglight verdant_froglight",
    ),
    (
        "c:gems",
        "amethyst_shard diamond emerald lapis_lazuli prismarine_crystals quartz",
    ),
    ("c:gems/amethyst", "amethyst_shard"),
    ("c:gems/diamond", "diamond"),
    ("c:gems/emerald", "emerald"),
    ("c:gems/lapis", "lapis_lazuli"),
    ("c:gems/prismarine", "prismarine_crystals"),
    ("c:gems/quartz", "quartz"),
    (
        "c:glass_blocks",
        "black_stained_glass blue_stained_glass brown_stained_glass cyan_stained_glass glass gray_stained_glass green_stained_glass light_blue_stained_glass light_gray_stained_glass lime_stained_glass magenta_stained_glass orange_stained_glass pink_stained_glass purple_stained_glass red_stained_glass tinted_glass white_stained_glass yellow_stained_glass",
    ),
    (
        "c:glass_blocks/cheap",
        "black_stained_glass blue_stained_glass brown_stained_glass cyan_stained_glass glass gray_stained_glass green_stained_glass light_blue_stained_glass light_gray_stained_glass lime_stained_glass magenta_stained_glass orange_stained_glass pink_stained_glass purple_stained_glass red_stained_glass white_stained_glass yellow_stained_glass",
    ),
    ("c:glass_blocks/colorless", "glass"),
    ("c:glass_blocks/tinted", "tinted_glass"),
    (
        "c:glass_panes",
        "black_stained_glass_pane blue_stained_glass_pane brown_stained_glass_pane cyan_stained_glass_pane glass_pane gray_stained_glass_pane green_stained_glass_pane light_blue_stained_glass_pane light_gray_stained_glass_pane lime_stained_glass_pane magenta_stained_glass_pane orange_stained_glass_pane pink_stained_glass_pane purple_stained_glass_pane red_stained_glass_pane white_stained_glass_pane yellow_stained_glass_pane",
    ),
    ("c:glass_panes/colorless", "glass_pane"),
    (
        "c:glazed_terracottas",
        "black_glazed_terracotta blue_glazed_terracotta brown_glazed_terracotta cyan_glazed_terracotta gray_glazed_terracotta green_glazed_terracotta light_blue_glazed_terracotta light_gray_glazed_terracotta lime_glazed_terracotta magenta_glazed_terracotta orange_glazed_terracotta pink_glazed_terracotta purple_glazed_terracotta red_glazed_terracotta white_glazed_terracotta yellow_glazed_terracotta",
    ),
    ("c:gravels", "gravel"),
    ("c:gunpowders", "gunpowder"),
    ("c:hidden_from_recipe_viewers", ""),
    ("c:ingots", "copper_ingot gold_ingot iron_ingot netherite_ingot"),
    ("c:ingots/copper", "copper_ingot"),
    ("c:ingots/gold", "gold_ingot"),
    ("c:ingots/iron", "iron_ingot"),
    ("c:ingots/netherite", "netherite_ingot"),
    ("c:leathers", "leather"),
    ("c:mushrooms", "brown_mushroom red_mushroom shelf_mushroom"),
    (
        "c:music_discs",
        "music_disc_11 music_disc_13 music_disc_5 music_disc_blocks music_disc_bounce music_disc_cat music_disc_chirp music_disc_creator music_disc_creator_music_box music_disc_far music_disc_lava_chicken music_disc_mall music_disc_mellohi music_disc_otherside music_disc_pigstep music_disc_precipice music_disc_relic music_disc_stal music_disc_strad music_disc_tears music_disc_wait music_disc_ward",
    ),
    (
        "c:natural_logs",
        "acacia_log bamboo_block birch_log cherry_log crimson_stem dark_oak_log jungle_log mangrove_log oak_log pale_oak_log poplar_log spruce_log warped_stem",
    ),
    ("c:natural_logs/nether", "crimson_stem warped_stem"),
    (
        "c:natural_logs/overworld",
        "acacia_log bamboo_block birch_log cherry_log dark_oak_log jungle_log mangrove_log oak_log pale_oak_log poplar_log spruce_log",
    ),
    (
        "c:natural_woods",
        "acacia_wood birch_wood cherry_wood crimson_hyphae dark_oak_wood jungle_wood mangrove_wood oak_wood pale_oak_wood poplar_wood spruce_wood warped_hyphae",
    ),
    ("c:nether_stars", "nether_star"),
    ("c:netherracks", "netherrack"),
    ("c:nuggets", "copper_nugget gold_nugget iron_nugget"),
    ("c:nuggets/copper", "copper_nugget"),
    ("c:nuggets/gold", "gold_nugget"),
    ("c:nuggets/iron", "iron_nugget"),
    ("c:obsidians", "crying_obsidian obsidian"),
    ("c:obsidians/crying", "crying_obsidian"),
    ("c:obsidians/normal", "obsidian"),
    ("c:ore_bearing_ground/deepslate", "deepslate"),
    ("c:ore_bearing_ground/netherrack", "netherrack"),
    ("c:ore_bearing_ground/stone", "stone"),
    (
        "c:ore_rates/dense",
        "copper_ore deepslate_copper_ore deepslate_lapis_ore deepslate_redstone_ore lapis_ore redstone_ore",
    ),
    (
        "c:ore_rates/singular",
        "ancient_debris coal_ore deepslate_coal_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore diamond_ore emerald_ore gold_ore iron_ore nether_quartz_ore",
    ),
    ("c:ore_rates/sparse", "nether_gold_ore"),
    (
        "c:ores",
        "ancient_debris coal_ore copper_ore deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore diamond_ore emerald_ore gold_ore iron_ore lapis_ore nether_gold_ore nether_quartz_ore redstone_ore",
    ),
    ("c:ores/coal", "coal_ore deepslate_coal_ore"),
    ("c:ores/copper", "copper_ore deepslate_copper_ore"),
    ("c:ores/diamond", "deepslate_diamond_ore diamond_ore"),
    ("c:ores/emerald", "deepslate_emerald_ore emerald_ore"),
    ("c:ores/gold", "deepslate_gold_ore gold_ore nether_gold_ore"),
    ("c:ores/iron", "deepslate_iron_ore iron_ore"),
    ("c:ores/lapis", "deepslate_lapis_ore lapis_ore"),
    ("c:ores/netherite_scrap", "ancient_debris"),
    ("c:ores/quartz", "nether_quartz_ore"),
    ("c:ores/redstone", "deepslate_redstone_ore redstone_ore"),
    (
        "c:ores_in_ground/deepslate",
        "deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore",
    ),
    ("c:ores_in_ground/netherrack", "nether_gold_ore nether_quartz_ore"),
    (
        "c:ores_in_ground/stone",
        "coal_ore copper_ore diamond_ore emerald_ore gold_ore iron_ore lapis_ore redstone_ore",
    ),
    ("c:player_workstations/crafting_tables", "crafting_table"),
    ("c:player_workstations/furnaces", "furnace"),
    ("c:potions", "lingering_potion potion splash_potion"),
    ("c:potions/bottle", "lingering_potion potion splash_potion"),
    ("c:pumpkins", "carved_pumpkin jack_o_lantern pumpkin"),
    ("c:pumpkins/carved", "carved_pumpkin"),
    ("c:pumpkins/jack_o_lanterns", "jack_o_lantern"),
    ("c:pumpkins/normal", "pumpkin"),
    ("c:raw_materials", "raw_copper raw_gold raw_iron"),
    ("c:raw_materials/copper", "raw_copper"),
    ("c:raw_materials/gold", "raw_gold"),
    ("c:raw_materials/iron", "raw_iron"),
    ("c:rods", "blaze_rod breeze_rod stick"),
    ("c:rods/blaze", "blaze_rod"),
    ("c:rods/breeze", "breeze_rod"),
    ("c:rods/wooden", "stick"),
    ("c:ropes", ""),
    ("c:sands", "red_sand sand"),
    ("c:sands/colorless", "sand"),
    ("c:sands/red", "red_sand"),
    (
        "c:sandstone/blocks",
        "chiseled_red_sandstone chiseled_sandstone cut_red_sandstone cut_sandstone red_sandstone sandstone smooth_red_sandstone smooth_sandstone",
    ),
    (
        "c:sandstone/red_blocks",
        "chiseled_red_sandstone cut_red_sandstone red_sandstone smooth_red_sandstone",
    ),
    (
        "c:sandstone/red_slabs",
        "cut_red_sandstone_slab red_sandstone_slab smooth_red_sandstone_slab",
    ),
    (
        "c:sandstone/red_stairs",
        "red_sandstone_stairs smooth_red_sandstone_stairs",
    ),
    (
        "c:sandstone/slabs",
        "cut_red_sandstone_slab cut_sandstone_slab red_sandstone_slab sandstone_slab smooth_red_sandstone_slab smooth_sandstone_slab",
    ),
    (
        "c:sandstone/stairs",
        "red_sandstone_stairs sandstone_stairs smooth_red_sandstone_stairs smooth_sandstone_stairs",
    ),
    (
        "c:sandstone/uncolored_blocks",
        "chiseled_sandstone cut_sandstone sandstone smooth_sandstone",
    ),
    (
        "c:sandstone/uncolored_slabs",
        "cut_sandstone_slab sandstone_slab smooth_sandstone_slab",
    ),
    (
        "c:sandstone/uncolored_stairs",
        "sandstone_stairs smooth_sandstone_stairs",
    ),
    (
        "c:seeds",
        "beetroot_seeds melon_seeds pitcher_pod pumpkin_seeds torchflower_seeds wheat_seeds",
    ),
    ("c:seeds/beetroot", "beetroot_seeds"),
    ("c:seeds/melon", "melon_seeds"),
    ("c:seeds/pitcher_plant", "pitcher_pod"),
    ("c:seeds/pumpkin", "pumpkin_seeds"),
    ("c:seeds/torchflower", "torchflower_seeds"),
    ("c:seeds/wheat", "wheat_seeds"),
    (
        "c:shulker_boxes",
        "black_shulker_box blue_shulker_box brown_shulker_box cyan_shulker_box gray_shulker_box green_shulker_box light_blue_shulker_box light_gray_shulker_box lime_shulker_box magenta_shulker_box orange_shulker_box pink_shulker_box purple_shulker_box red_shulker_box shulker_box white_shulker_box yellow_shulker_box",
    ),
    ("c:slime_balls", "slime_ball"),
    ("c:stones", "andesite deepslate diorite granite stone tuff"),
    (
        "c:storage_blocks",
        "bone_block coal_block copper_block diamond_block dried_kelp_block emerald_block gold_block hay_block iron_block lapis_block netherite_block raw_copper_block raw_gold_block raw_iron_block redstone_block resin_block slime_block",
    ),
    ("c:storage_blocks/bone_meal", "bone_block"),
    ("c:storage_blocks/coal", "coal_block"),
    ("c:storage_blocks/copper", "copper_block"),
    ("c:storage_blocks/diamond", "diamond_block"),
    ("c:storage_blocks/dried_kelp", "dried_kelp_block"),
    ("c:storage_blocks/emerald", "emerald_block"),
    ("c:storage_blocks/gold", "gold_block"),
    ("c:storage_blocks/iron", "iron_block"),
    ("c:storage_blocks/lapis", "lapis_block"),
    ("c:storage_blocks/netherite", "netherite_block"),
    ("c:storage_blocks/raw_copper", "raw_copper_block"),
    ("c:storage_blocks/raw_gold", "raw_gold_block"),
    ("c:storage_blocks/raw_iron", "raw_iron_block"),
    ("c:storage_blocks/redstone", "redstone_block"),
    ("c:storage_blocks/resin", "resin_block"),
    ("c:storage_blocks/slime", "slime_block"),
    ("c:storage_blocks/wheat", "hay_block"),
    ("c:strings", "string"),
    (
        "c:stripped_logs",
        "stripped_acacia_log stripped_bamboo_block stripped_birch_log stripped_cherry_log stripped_crimson_stem stripped_dark_oak_log stripped_jungle_log stripped_mangrove_log stripped_oak_log stripped_pale_oak_log stripped_poplar_log stripped_spruce_log stripped_warped_stem",
    ),
    (
        "c:stripped_woods",
        "stripped_acacia_wood stripped_birch_wood stripped_cherry_wood stripped_crimson_hyphae stripped_dark_oak_wood stripped_jungle_wood stripped_mangrove_wood stripped_oak_wood stripped_pale_oak_wood stripped_poplar_wood stripped_spruce_wood stripped_warped_hyphae",
    ),
    (
        "c:tools",
        "bow brush copper_axe copper_hoe copper_pickaxe copper_shovel copper_spear copper_sword crossbow diamond_axe diamond_hoe diamond_pickaxe diamond_shovel diamond_spear diamond_sword fishing_rod flint_and_steel golden_axe golden_hoe golden_pickaxe golden_shovel golden_spear golden_sword iron_axe iron_hoe iron_pickaxe iron_shovel iron_spear iron_sword mace netherite_axe netherite_hoe netherite_pickaxe netherite_shovel netherite_spear netherite_sword shears shield stone_axe stone_hoe stone_pickaxe stone_shovel stone_spear stone_sword trident wooden_axe wooden_hoe wooden_pickaxe wooden_shovel wooden_spear wooden_sword",
    ),
    ("c:tools/bow", "bow"),
    ("c:tools/brush", "brush"),
    ("c:tools/crossbow", "crossbow"),
    ("c:tools/fishing_rod", "fishing_rod"),
    ("c:tools/igniter", "flint_and_steel"),
    ("c:tools/mace", "mace"),
    (
        "c:tools/melee_weapon",
        "copper_axe copper_spear copper_sword diamond_axe diamond_spear diamond_sword golden_axe golden_spear golden_sword iron_axe iron_spear iron_sword mace netherite_axe netherite_spear netherite_sword stone_axe stone_spear stone_sword trident wooden_axe wooden_spear wooden_sword",
    ),
    (
        "c:tools/mining_tool",
        "copper_pickaxe diamond_pickaxe golden_pickaxe iron_pickaxe netherite_pickaxe stone_pickaxe wooden_pickaxe",
    ),
    ("c:tools/ranged_weapon", "bow crossbow trident"),
    ("c:tools/shear", "shears"),
    ("c:tools/shield", "shield"),
    ("c:tools/trident", "trident"),
    ("c:tools/wrench", ""),
    (
        "c:villager_job_sites",
        "barrel blast_furnace brewing_stand cartography_table cauldron composter fletching_table grindstone lectern loom smithing_table smoker stonecutter",
    ),
    (
        "minecraft:acacia_logs",
        "acacia_log acacia_wood stripped_acacia_log stripped_acacia_wood",
    ),
    ("minecraft:anvil", "anvil chipped_anvil damaged_anvil"),
    ("minecraft:armadillo_food", "spider_eye"),
    ("minecraft:arrows", "arrow spectral_arrow tipped_arrow"),
    (
        "minecraft:axes",
        "copper_axe diamond_axe golden_axe iron_axe netherite_axe stone_axe wooden_axe",
    ),
    ("minecraft:axolotl_food", "tropical_fish_bucket"),
    ("minecraft:bamboo_blocks", "bamboo_block stripped_bamboo_block"),
    (
        "minecraft:banners",
        "black_banner blue_banner brown_banner cyan_banner gray_banner green_banner light_blue_banner light_gray_banner lime_banner magenta_banner orange_banner pink_banner purple_banner red_banner white_banner yellow_banner",
    ),
    (
        "minecraft:bars",
        "copper_bars exposed_copper_bars iron_bars oxidized_copper_bars waxed_copper_bars waxed_exposed_copper_bars waxed_oxidized_copper_bars waxed_weathered_copper_bars weathered_copper_bars",
    ),
    (
        "minecraft:beacon_payment_items",
        "diamond emerald gold_ingot iron_ingot netherite_ingot",
    ),
    (
        "minecraft:beds",
        "black_bed blue_bed brown_bed cyan_bed gray_bed green_bed light_blue_bed light_gray_bed lime_bed magenta_bed orange_bed pink_bed purple_bed red_bed white_bed yellow_bed",
    ),
    (
        "minecraft:bee_food",
        "allium azure_bluet blue_orchid cactus_flower cherry_leaves chorus_flower cornflower dandelion flowering_azalea flowering_azalea_leaves lilac lily_of_the_valley mangrove_propagule open_eyeblossom orange_tulip oxeye_daisy peony pink_petals pink_tulip pitcher_plant poppy red_tulip rose_bush spore_blossom sunflower torchflower white_tulip wildflowers wither_rose",
    ),
    (
        "minecraft:birch_logs",
        "birch_log birch_wood stripped_birch_log stripped_birch_wood",
    ),
    (
        "minecraft:boats",
        "acacia_boat acacia_chest_boat bamboo_chest_raft bamboo_raft birch_boat birch_chest_boat cherry_boat cherry_chest_boat dark_oak_boat dark_oak_chest_boat jungle_boat jungle_chest_boat mangrove_boat mangrove_chest_boat oak_boat oak_chest_boat pale_oak_boat pale_oak_chest_boat poplar_boat poplar_chest_boat spruce_boat spruce_chest_boat",
    ),
    ("minecraft:book_cloning_target", "writable_book"),
    (
        "minecraft:bookshelf_books",
        "book enchanted_book knowledge_book writable_book written_book",
    ),
    (
        "minecraft:breaks_decorated_pots",
        "copper_axe copper_hoe copper_pickaxe copper_shovel copper_sword diamond_axe diamond_hoe diamond_pickaxe diamond_shovel diamond_sword golden_axe golden_hoe golden_pickaxe golden_shovel golden_sword iron_axe iron_hoe iron_pickaxe iron_shovel iron_sword mace netherite_axe netherite_hoe netherite_pickaxe netherite_shovel netherite_sword stone_axe stone_hoe stone_pickaxe stone_shovel stone_sword trident wooden_axe wooden_hoe wooden_pickaxe wooden_shovel wooden_sword",
    ),
    (
        "minecraft:brewing_potion_inputs",
        "glass_bottle lingering_potion potion splash_potion",
    ),
    (
        "minecraft:bundles",
        "black_bundle blue_bundle brown_bundle bundle cyan_bundle gray_bundle green_bundle light_blue_bundle light_gray_bundle lime_bundle magenta_bundle orange_bundle pink_bundle purple_bundle red_bundle white_bundle yellow_bundle",
    ),
    (
        "minecraft:buttons",
        "acacia_button bamboo_button birch_button cherry_button crimson_button dark_oak_button jungle_button mangrove_button oak_button pale_oak_button polished_blackstone_button poplar_button spruce_button stone_button warped_button",
    ),
    ("minecraft:camel_food", "cactus"),
    ("minecraft:camel_husk_food", "rabbit_foot"),
    (
        "minecraft:candles",
        "black_candle blue_candle brown_candle candle cyan_candle gray_candle green_candle light_blue_candle light_gray_candle lime_candle magenta_candle orange_candle pink_candle purple_candle red_candle white_candle yellow_candle",
    ),
    (
        "minecraft:cat_collar_dyes",
        "black_dye blue_dye brown_dye cyan_dye gray_dye green_dye light_blue_dye light_gray_dye lime_dye magenta_dye orange_dye pink_dye purple_dye red_dye white_dye yellow_dye",
    ),
    ("minecraft:cat_food", "cod salmon"),
    (
        "minecraft:cauldron_can_remove_dye",
        "leather_boots leather_chestplate leather_helmet leather_horse_armor leather_leggings wolf_armor",
    ),
    (
        "minecraft:chains",
        "copper_chain exposed_copper_chain iron_chain oxidized_copper_chain waxed_copper_chain waxed_exposed_copper_chain waxed_oxidized_copper_chain waxed_weathered_copper_chain weathered_copper_chain",
    ),
    (
        "minecraft:cherry_logs",
        "cherry_log cherry_wood stripped_cherry_log stripped_cherry_wood",
    ),
    (
        "minecraft:chest_armor",
        "chainmail_chestplate copper_chestplate diamond_chestplate golden_chestplate iron_chestplate leather_chestplate netherite_chestplate",
    ),
    (
        "minecraft:chest_boats",
        "acacia_chest_boat bamboo_chest_raft birch_chest_boat cherry_chest_boat dark_oak_chest_boat jungle_chest_boat mangrove_chest_boat oak_chest_boat pale_oak_chest_boat poplar_chest_boat spruce_chest_boat",
    ),
    (
        "minecraft:chicken_food",
        "beetroot_seeds melon_seeds pitcher_pod pumpkin_seeds torchflower_seeds wheat_seeds",
    ),
    (
        "minecraft:clonable_maps",
        "abandoned_camp_map buried_ancient_city_map buried_mineshaft_map buried_treasure_map buried_trial_chambers_map desert_pyramid_map desert_village_map filled_map jungle_pyramid_map ocean_monument_map plains_village_map savanna_village_map snowy_village_map swamp_hut_map taiga_village_map warm_ocean_ruins_map woodland_mansion_map",
    ),
    (
        "minecraft:cluster_max_harvestables",
        "copper_pickaxe diamond_pickaxe golden_pickaxe iron_pickaxe netherite_pickaxe stone_pickaxe wooden_pickaxe",
    ),
    ("minecraft:coal_ores", "coal_ore deepslate_coal_ore"),
    ("minecraft:coals", "charcoal coal"),
    ("minecraft:compasses", "compass recovery_compass"),
    (
        "minecraft:completes_find_tree_tutorial",
        "acacia_leaves acacia_log acacia_wood azalea_leaves birch_leaves birch_log birch_wood cherry_leaves cherry_log cherry_wood crimson_hyphae crimson_stem dark_oak_leaves dark_oak_log dark_oak_wood flowering_azalea_leaves jungle_leaves jungle_log jungle_wood mangrove_leaves mangrove_log mangrove_wood nether_wart_block oak_leaves oak_log oak_wood orange_poplar_leaves pale_oak_leaves pale_oak_log pale_oak_wood poplar_log poplar_wood red_poplar_leaves spruce_leaves spruce_log spruce_wood stripped_acacia_log stripped_acacia_wood stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem warped_hyphae warped_stem warped_wart_block yellow_poplar_leaves",
    ),
    (
        "minecraft:concrete",
        "black_concrete blue_concrete brown_concrete cyan_concrete gray_concrete green_concrete light_blue_concrete light_gray_concrete lime_concrete magenta_concrete orange_concrete pink_concrete purple_concrete red_concrete white_concrete yellow_concrete",
    ),
    (
        "minecraft:concrete_powders",
        "black_concrete_powder blue_concrete_powder brown_concrete_powder cyan_concrete_powder gray_concrete_powder green_concrete_powder light_blue_concrete_powder light_gray_concrete_powder lime_concrete_powder magenta_concrete_powder orange_concrete_powder pink_concrete_powder purple_concrete_powder red_concrete_powder white_concrete_powder yellow_concrete_powder",
    ),
    (
        "minecraft:concrete_slabs",
        "black_concrete_slab blue_concrete_slab brown_concrete_slab cyan_concrete_slab gray_concrete_slab green_concrete_slab light_blue_concrete_slab light_gray_concrete_slab lime_concrete_slab magenta_concrete_slab orange_concrete_slab pink_concrete_slab purple_concrete_slab red_concrete_slab white_concrete_slab yellow_concrete_slab",
    ),
    (
        "minecraft:concrete_stairs",
        "black_concrete_stairs blue_concrete_stairs brown_concrete_stairs cyan_concrete_stairs gray_concrete_stairs green_concrete_stairs light_blue_concrete_stairs light_gray_concrete_stairs lime_concrete_stairs magenta_concrete_stairs orange_concrete_stairs pink_concrete_stairs purple_concrete_stairs red_concrete_stairs white_concrete_stairs yellow_concrete_stairs",
    ),
    (
        "minecraft:copper",
        "copper_block exposed_copper oxidized_copper waxed_copper_block waxed_exposed_copper waxed_oxidized_copper waxed_weathered_copper weathered_copper",
    ),
    (
        "minecraft:copper_chests",
        "copper_chest exposed_copper_chest oxidized_copper_chest waxed_copper_chest waxed_exposed_copper_chest waxed_oxidized_copper_chest waxed_weathered_copper_chest weathered_copper_chest",
    ),
    (
        "minecraft:copper_golem_statues",
        "copper_golem_statue exposed_copper_golem_statue oxidized_copper_golem_statue waxed_copper_golem_statue waxed_exposed_copper_golem_statue waxed_oxidized_copper_golem_statue waxed_weathered_copper_golem_statue weathered_copper_golem_statue",
    ),
    ("minecraft:copper_ores", "copper_ore deepslate_copper_ore"),
    ("minecraft:copper_tool_materials", "copper_ingot"),
    ("minecraft:cow_food", "wheat"),
    (
        "minecraft:creeper_drop_music_discs",
        "music_disc_11 music_disc_13 music_disc_blocks music_disc_cat music_disc_chirp music_disc_far music_disc_mall music_disc_mellohi music_disc_stal music_disc_strad music_disc_wait music_disc_ward",
    ),
    ("minecraft:creeper_igniters", "fire_charge flint_and_steel"),
    (
        "minecraft:crimson_stems",
        "crimson_hyphae crimson_stem stripped_crimson_hyphae stripped_crimson_stem",
    ),
    (
        "minecraft:cushions",
        "black_cushion blue_cushion brown_cushion cyan_cushion gray_cushion green_cushion light_blue_cushion light_gray_cushion lime_cushion magenta_cushion orange_cushion pink_cushion purple_cushion red_cushion white_cushion yellow_cushion",
    ),
    (
        "minecraft:dampens_vibrations",
        "black_carpet black_wool black_wool_slab black_wool_stairs blue_carpet blue_wool blue_wool_slab blue_wool_stairs brown_carpet brown_wool brown_wool_slab brown_wool_stairs cyan_carpet cyan_wool cyan_wool_slab cyan_wool_stairs gray_carpet gray_wool gray_wool_slab gray_wool_stairs green_carpet green_wool green_wool_slab green_wool_stairs light_blue_carpet light_blue_wool light_blue_wool_slab light_blue_wool_stairs light_gray_carpet light_gray_wool light_gray_wool_slab light_gray_wool_stairs lime_carpet lime_wool lime_wool_slab lime_wool_stairs magenta_carpet magenta_wool magenta_wool_slab magenta_wool_stairs orange_carpet orange_wool orange_wool_slab orange_wool_stairs pink_carpet pink_wool pink_wool_slab pink_wool_stairs purple_carpet purple_wool purple_wool_slab purple_wool_stairs red_carpet red_wool red_wool_slab red_wool_stairs white_carpet white_wool white_wool_slab white_wool_stairs yellow_carpet yellow_wool yellow_wool_slab yellow_wool_stairs",
    ),
    (
        "minecraft:dark_oak_logs",
        "dark_oak_log dark_oak_wood stripped_dark_oak_log stripped_dark_oak_wood",
    ),
    (
        "minecraft:decorated_pot_ingredients",
        "angler_pottery_sherd archer_pottery_sherd arms_up_pottery_sherd blade_pottery_sherd brewer_pottery_sherd brick burn_pottery_sherd danger_pottery_sherd explorer_pottery_sherd flow_pottery_sherd friend_pottery_sherd guster_pottery_sherd heart_pottery_sherd heartbreak_pottery_sherd howl_pottery_sherd miner_pottery_sherd mourner_pottery_sherd plenty_pottery_sherd prize_pottery_sherd scrape_pottery_sherd sheaf_pottery_sherd shelter_pottery_sherd skull_pottery_sherd snort_pottery_sherd",
    ),
    (
        "minecraft:decorated_pot_sherds",
        "angler_pottery_sherd archer_pottery_sherd arms_up_pottery_sherd blade_pottery_sherd brewer_pottery_sherd burn_pottery_sherd danger_pottery_sherd explorer_pottery_sherd flow_pottery_sherd friend_pottery_sherd guster_pottery_sherd heart_pottery_sherd heartbreak_pottery_sherd howl_pottery_sherd miner_pottery_sherd mourner_pottery_sherd plenty_pottery_sherd prize_pottery_sherd scrape_pottery_sherd sheaf_pottery_sherd shelter_pottery_sherd skull_pottery_sherd snort_pottery_sherd",
    ),
    ("minecraft:diamond_ores", "deepslate_diamond_ore diamond_ore"),
    ("minecraft:diamond_tool_materials", "diamond"),
    ("minecraft:dirt", "coarse_dirt dirt rooted_dirt"),
    (
        "minecraft:doors",
        "acacia_door bamboo_door birch_door cherry_door copper_door crimson_door dark_oak_door exposed_copper_door iron_door jungle_door mangrove_door oak_door oxidized_copper_door pale_oak_door poplar_door spruce_door warped_door waxed_copper_door waxed_exposed_copper_door waxed_oxidized_copper_door waxed_weathered_copper_door weathered_copper_door",
    ),
    (
        "minecraft:douses_campfires",
        "copper_shovel diamond_shovel golden_shovel iron_shovel netherite_shovel stone_shovel wooden_shovel",
    ),
    ("minecraft:drowned_preferred_weapons", "trident"),
    ("minecraft:duplicates_allays", "amethyst_shard"),
    (
        "minecraft:dyes",
        "black_dye blue_dye brown_dye cyan_dye gray_dye green_dye light_blue_dye light_gray_dye lime_dye magenta_dye orange_dye pink_dye purple_dye red_dye white_dye yellow_dye",
    ),
    ("minecraft:eggs", "blue_egg brown_egg egg"),
    ("minecraft:emerald_ores", "deepslate_emerald_ore emerald_ore"),
    (
        "minecraft:enchantable/armor",
        "chainmail_boots chainmail_chestplate chainmail_helmet chainmail_leggings copper_boots copper_chestplate copper_helmet copper_leggings diamond_boots diamond_chestplate diamond_helmet diamond_leggings golden_boots golden_chestplate golden_helmet golden_leggings iron_boots iron_chestplate iron_helmet iron_leggings leather_boots leather_chestplate leather_helmet leather_leggings netherite_boots netherite_chestplate netherite_helmet netherite_leggings turtle_helmet",
    ),
    ("minecraft:enchantable/bow", "bow"),
    (
        "minecraft:enchantable/chest_armor",
        "chainmail_chestplate copper_chestplate diamond_chestplate golden_chestplate iron_chestplate leather_chestplate netherite_chestplate",
    ),
    ("minecraft:enchantable/crossbow", "crossbow"),
    (
        "minecraft:enchantable/durability",
        "bow brush carrot_on_a_stick chainmail_boots chainmail_chestplate chainmail_helmet chainmail_leggings copper_axe copper_boots copper_chestplate copper_helmet copper_hoe copper_leggings copper_pickaxe copper_shovel copper_spear copper_sword crossbow diamond_axe diamond_boots diamond_chestplate diamond_helmet diamond_hoe diamond_leggings diamond_pickaxe diamond_shovel diamond_spear diamond_sword elytra fishing_rod flint_and_steel golden_axe golden_boots golden_chestplate golden_helmet golden_hoe golden_leggings golden_pickaxe golden_shovel golden_spear golden_sword iron_axe iron_boots iron_chestplate iron_helmet iron_hoe iron_leggings iron_pickaxe iron_shovel iron_spear iron_sword leather_boots leather_chestplate leather_helmet leather_leggings mace netherite_axe netherite_boots netherite_chestplate netherite_helmet netherite_hoe netherite_leggings netherite_pickaxe netherite_shovel netherite_spear netherite_sword shears shield stone_axe stone_hoe stone_pickaxe stone_shovel stone_spear stone_sword trident turtle_helmet warped_fungus_on_a_stick wooden_axe wooden_hoe wooden_pickaxe wooden_shovel wooden_spear wooden_sword",
    ),
    (
        "minecraft:enchantable/equippable",
        "carved_pumpkin chainmail_boots chainmail_chestplate chainmail_helmet chainmail_leggings copper_boots copper_chestplate copper_helmet copper_leggings creeper_head diamond_boots diamond_chestplate diamond_helmet diamond_leggings dragon_head elytra golden_boots golden_chestplate golden_helmet golden_leggings iron_boots iron_chestplate iron_helmet iron_leggings leather_boots leather_chestplate leather_helmet leather_leggings netherite_boots netherite_chestplate netherite_helmet netherite_leggings piglin_head player_head skeleton_skull turtle_helmet wither_skeleton_skull zombie_head",
    ),
    (
        "minecraft:enchantable/fire_aspect",
        "copper_spear copper_sword diamond_spear diamond_sword golden_spear golden_sword iron_spear iron_sword mace netherite_spear netherite_sword stone_spear stone_sword wooden_spear wooden_sword",
    ),
    ("minecraft:enchantable/fishing", "fishing_rod"),
    (
        "minecraft:enchantable/foot_armor",
        "chainmail_boots copper_boots diamond_boots golden_boots iron_boots leather_boots netherite_boots",
    ),
    (
        "minecraft:enchantable/head_armor",
        "chainmail_helmet copper_helmet diamond_helmet golden_helmet iron_helmet leather_helmet netherite_helmet turtle_helmet",
    ),
    (
        "minecraft:enchantable/leg_armor",
        "chainmail_leggings copper_leggings diamond_leggings golden_leggings iron_leggings leather_leggings netherite_leggings",
    ),
    (
        "minecraft:enchantable/lunge",
        "copper_spear diamond_spear golden_spear iron_spear netherite_spear stone_spear wooden_spear",
    ),
    ("minecraft:enchantable/mace", "mace"),
    (
        "minecraft:enchantable/melee_weapon",
        "copper_spear copper_sword diamond_spear diamond_sword golden_spear golden_sword iron_spear iron_sword netherite_spear netherite_sword stone_spear stone_sword wooden_spear wooden_sword",
    ),
    (
        "minecraft:enchantable/mining",
        "copper_axe copper_hoe copper_pickaxe copper_shovel diamond_axe diamond_hoe diamond_pickaxe diamond_shovel golden_axe golden_hoe golden_pickaxe golden_shovel iron_axe iron_hoe iron_pickaxe iron_shovel netherite_axe netherite_hoe netherite_pickaxe netherite_shovel shears stone_axe stone_hoe stone_pickaxe stone_shovel wooden_axe wooden_hoe wooden_pickaxe wooden_shovel",
    ),
    (
        "minecraft:enchantable/mining_loot",
        "copper_axe copper_hoe copper_pickaxe copper_shovel diamond_axe diamond_hoe diamond_pickaxe diamond_shovel golden_axe golden_hoe golden_pickaxe golden_shovel iron_axe iron_hoe iron_pickaxe iron_shovel netherite_axe netherite_hoe netherite_pickaxe netherite_shovel stone_axe stone_hoe stone_pickaxe stone_shovel wooden_axe wooden_hoe wooden_pickaxe wooden_shovel",
    ),
    (
        "minecraft:enchantable/sharp_weapon",
        "copper_axe copper_spear copper_sword diamond_axe diamond_spear diamond_sword golden_axe golden_spear golden_sword iron_axe iron_spear iron_sword netherite_axe netherite_spear netherite_sword stone_axe stone_spear stone_sword wooden_axe wooden_spear wooden_sword",
    ),
    (
        "minecraft:enchantable/sweeping",
        "copper_sword diamond_sword golden_sword iron_sword netherite_sword stone_sword wooden_sword",
    ),
    ("minecraft:enchantable/trident", "trident"),
    (
        "minecraft:enchantable/vanishing",
        "bow brush carrot_on_a_stick carved_pumpkin chainmail_boots chainmail_chestplate chainmail_helmet chainmail_leggings compass copper_axe copper_boots copper_chestplate copper_helmet copper_hoe copper_leggings copper_pickaxe copper_shovel copper_spear copper_sword creeper_head crossbow diamond_axe diamond_boots diamond_chestplate diamond_helmet diamond_hoe diamond_leggings diamond_pickaxe diamond_shovel diamond_spear diamond_sword dragon_head elytra fishing_rod flint_and_steel golden_axe golden_boots golden_chestplate golden_helmet golden_hoe golden_leggings golden_pickaxe golden_shovel golden_spear golden_sword iron_axe iron_boots iron_chestplate iron_helmet iron_hoe iron_leggings iron_pickaxe iron_shovel iron_spear iron_sword leather_boots leather_chestplate leather_helmet leather_leggings mace netherite_axe netherite_boots netherite_chestplate netherite_helmet netherite_hoe netherite_leggings netherite_pickaxe netherite_shovel netherite_spear netherite_sword piglin_head player_head shears shield skeleton_skull stone_axe stone_hoe stone_pickaxe stone_shovel stone_spear stone_sword trident turtle_helmet warped_fungus_on_a_stick wither_skeleton_skull wooden_axe wooden_hoe wooden_pickaxe wooden_shovel wooden_spear wooden_sword zombie_head",
    ),
    (
        "minecraft:enchantable/weapon",
        "copper_axe copper_spear copper_sword diamond_axe diamond_spear diamond_sword golden_axe golden_spear golden_sword iron_axe iron_spear iron_sword mace netherite_axe netherite_spear netherite_sword stone_axe stone_spear stone_sword wooden_axe wooden_spear wooden_sword",
    ),
    ("minecraft:extendable_maps", "filled_map"),
    (
        "minecraft:fence_gates",
        "acacia_fence_gate bamboo_fence_gate birch_fence_gate cherry_fence_gate crimson_fence_gate dark_oak_fence_gate jungle_fence_gate mangrove_fence_gate oak_fence_gate pale_oak_fence_gate poplar_fence_gate spruce_fence_gate warped_fence_gate",
    ),
    (
        "minecraft:fences",
        "acacia_fence bamboo_fence birch_fence cherry_fence crimson_fence dark_oak_fence jungle_fence mangrove_fence nether_brick_fence oak_fence pale_oak_fence poplar_fence spruce_fence warped_fence",
    ),
    (
        "minecraft:fishes",
        "cod cooked_cod cooked_salmon pufferfish salmon tropical_fish",
    ),
    (
        "minecraft:flowers",
        "allium azure_bluet blue_orchid cactus_flower cherry_leaves chorus_flower closed_eyeblossom cornflower dandelion flowering_azalea flowering_azalea_leaves golden_dandelion lilac lily_of_the_valley mangrove_propagule open_eyeblossom orange_tulip oxeye_daisy peony pink_petals pink_tulip pitcher_plant poppy red_tulip rose_bush spore_blossom sunflower torchflower white_tulip wildflowers wither_rose",
    ),
    (
        "minecraft:foot_armor",
        "chainmail_boots copper_boots diamond_boots golden_boots iron_boots leather_boots netherite_boots",
    ),
    ("minecraft:fox_food", "glow_berries sweet_berries"),
    (
        "minecraft:freeze_immune_wearables",
        "leather_boots leather_chestplate leather_helmet leather_horse_armor leather_leggings",
    ),
    ("minecraft:frog_food", "slime_ball"),
    ("minecraft:furnace_fuel_bottom_takeable", "bucket water_bucket"),
    ("minecraft:furnace_minecart_fuel", "charcoal coal"),
    ("minecraft:gaze_disguise_equipment", "carved_pumpkin"),
    (
        "minecraft:glazed_terracotta",
        "black_glazed_terracotta blue_glazed_terracotta brown_glazed_terracotta cyan_glazed_terracotta gray_glazed_terracotta green_glazed_terracotta light_blue_glazed_terracotta light_gray_glazed_terracotta lime_glazed_terracotta magenta_glazed_terracotta orange_glazed_terracotta pink_glazed_terracotta purple_glazed_terracotta red_glazed_terracotta white_glazed_terracotta yellow_glazed_terracotta",
    ),
    ("minecraft:goat_food", "wheat"),
    (
        "minecraft:gold_ores",
        "deepslate_gold_ore gold_ore nether_gold_ore",
    ),
    ("minecraft:gold_tool_materials", "gold_ingot"),
    ("minecraft:grass_blocks", "grass_block mycelium podzol"),
    (
        "minecraft:hanging_signs",
        "acacia_hanging_sign bamboo_hanging_sign birch_hanging_sign cherry_hanging_sign crimson_hanging_sign dark_oak_hanging_sign jungle_hanging_sign mangrove_hanging_sign oak_hanging_sign pale_oak_hanging_sign poplar_hanging_sign spruce_hanging_sign warped_hanging_sign",
    ),
    ("minecraft:happy_ghast_food", "snowball"),
    (
        "minecraft:happy_ghast_tempt_items",
        "black_harness blue_harness brown_harness cyan_harness gray_harness green_harness light_blue_harness light_gray_harness lime_harness magenta_harness orange_harness pink_harness purple_harness red_harness snowball white_harness yellow_harness",
    ),
    (
        "minecraft:harnesses",
        "black_harness blue_harness brown_harness cyan_harness gray_harness green_harness light_blue_harness light_gray_harness lime_harness magenta_harness orange_harness pink_harness purple_harness red_harness white_harness yellow_harness",
    ),
    (
        "minecraft:head_armor",
        "chainmail_helmet copper_helmet diamond_helmet golden_helmet iron_helmet leather_helmet netherite_helmet turtle_helmet",
    ),
    (
        "minecraft:hoes",
        "copper_hoe diamond_hoe golden_hoe iron_hoe netherite_hoe stone_hoe wooden_hoe",
    ),
    ("minecraft:hoglin_food", "crimson_fungus"),
    (
        "minecraft:horse_food",
        "apple carrot enchanted_golden_apple golden_apple golden_carrot hay_block sugar wheat",
    ),
    (
        "minecraft:horse_tempt_items",
        "enchanted_golden_apple golden_apple golden_carrot",
    ),
    ("minecraft:ignored_by_piglin_babies", "leather"),
    ("minecraft:iron_ores", "deepslate_iron_ore iron_ore"),
    ("minecraft:iron_tool_materials", "iron_ingot"),
    (
        "minecraft:jungle_logs",
        "jungle_log jungle_wood stripped_jungle_log stripped_jungle_wood",
    ),
    (
        "minecraft:lanterns",
        "copper_lantern exposed_copper_lantern lantern oxidized_copper_lantern soul_lantern waxed_copper_lantern waxed_exposed_copper_lantern waxed_oxidized_copper_lantern waxed_weathered_copper_lantern weathered_copper_lantern",
    ),
    ("minecraft:lapis_ores", "deepslate_lapis_ore lapis_ore"),
    (
        "minecraft:leaves",
        "acacia_leaves azalea_leaves birch_leaves cherry_leaves dark_oak_leaves flowering_azalea_leaves jungle_leaves mangrove_leaves oak_leaves orange_poplar_leaves pale_oak_leaves red_poplar_leaves spruce_leaves yellow_poplar_leaves",
    ),
    ("minecraft:lectern_books", "writable_book written_book"),
    (
        "minecraft:leg_armor",
        "chainmail_leggings copper_leggings diamond_leggings golden_leggings iron_leggings leather_leggings netherite_leggings",
    ),
    (
        "minecraft:lightning_rods",
        "exposed_lightning_rod lightning_rod oxidized_lightning_rod waxed_exposed_lightning_rod waxed_lightning_rod waxed_oxidized_lightning_rod waxed_weathered_lightning_rod weathered_lightning_rod",
    ),
    ("minecraft:llama_food", "hay_block wheat"),
    ("minecraft:llama_tempt_items", "hay_block"),
    (
        "minecraft:logs",
        "acacia_log acacia_wood birch_log birch_wood cherry_log cherry_wood crimson_hyphae crimson_stem dark_oak_log dark_oak_wood jungle_log jungle_wood mangrove_log mangrove_wood oak_log oak_wood pale_oak_log pale_oak_wood poplar_log poplar_wood spruce_log spruce_wood stripped_acacia_log stripped_acacia_wood stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem warped_hyphae warped_stem",
    ),
    (
        "minecraft:logs_that_burn",
        "acacia_log acacia_wood birch_log birch_wood cherry_log cherry_wood dark_oak_log dark_oak_wood jungle_log jungle_wood mangrove_log mangrove_wood oak_log oak_wood pale_oak_log pale_oak_wood poplar_log poplar_wood spruce_log spruce_wood stripped_acacia_log stripped_acacia_wood stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood",
    ),
    (
        "minecraft:loom_dyes",
        "black_dye blue_dye brown_dye cyan_dye gray_dye green_dye light_blue_dye light_gray_dye lime_dye magenta_dye orange_dye pink_dye purple_dye red_dye white_dye yellow_dye",
    ),
    (
        "minecraft:loom_patterns",
        "bordure_indented_banner_pattern creeper_banner_pattern field_masoned_banner_pattern flow_banner_pattern flower_banner_pattern globe_banner_pattern guster_banner_pattern mojang_banner_pattern piglin_banner_pattern skull_banner_pattern",
    ),
    (
        "minecraft:mangrove_logs",
        "mangrove_log mangrove_wood stripped_mangrove_log stripped_mangrove_wood",
    ),
    ("minecraft:map_invisibility_equipment", "carved_pumpkin"),
    (
        "minecraft:meat",
        "beef chicken cooked_beef cooked_chicken cooked_mutton cooked_porkchop cooked_rabbit mutton porkchop rabbit rotten_flesh",
    ),
    ("minecraft:metal_nuggets", "copper_nugget gold_nugget iron_nugget"),
    ("minecraft:moss_blocks", "moss_block pale_moss_block"),
    ("minecraft:mud", "mud muddy_mangrove_roots"),
    (
        "minecraft:mushrooms",
        "brown_mushroom red_mushroom shelf_mushroom",
    ),
    (
        "minecraft:nautilus_bucket_food",
        "cod_bucket pufferfish_bucket salmon_bucket tropical_fish_bucket",
    ),
    (
        "minecraft:nautilus_food",
        "cod cod_bucket cooked_cod cooked_salmon pufferfish pufferfish_bucket salmon salmon_bucket tropical_fish tropical_fish_bucket",
    ),
    ("minecraft:nautilus_taming_items", "pufferfish pufferfish_bucket"),
    ("minecraft:netherite_tool_materials", "netherite_ingot"),
    (
        "minecraft:non_flammable_wood",
        "crimson_button crimson_door crimson_fence crimson_fence_gate crimson_hanging_sign crimson_hyphae crimson_planks crimson_pressure_plate crimson_shelf crimson_sign crimson_slab crimson_stairs crimson_stem crimson_trapdoor stripped_crimson_hyphae stripped_crimson_stem stripped_warped_hyphae stripped_warped_stem warped_button warped_door warped_fence warped_fence_gate warped_hanging_sign warped_hyphae warped_planks warped_pressure_plate warped_shelf warped_sign warped_slab warped_stairs warped_stem warped_trapdoor",
    ),
    (
        "minecraft:noteblock_top_instruments",
        "creeper_head dragon_head piglin_head player_head skeleton_skull wither_skeleton_skull zombie_head",
    ),
    (
        "minecraft:oak_logs",
        "oak_log oak_wood stripped_oak_log stripped_oak_wood",
    ),
    ("minecraft:ocelot_food", "cod salmon"),
    (
        "minecraft:ores",
        "coal_ore copper_ore deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore diamond_ore emerald_ore gold_ore iron_ore lapis_ore nether_gold_ore nether_quartz_ore redstone_ore",
    ),
    (
        "minecraft:pale_oak_logs",
        "pale_oak_log pale_oak_wood stripped_pale_oak_log stripped_pale_oak_wood",
    ),
    ("minecraft:panda_eats_from_ground", "bamboo cake"),
    ("minecraft:panda_food", "bamboo"),
    (
        "minecraft:parrot_food",
        "beetroot_seeds melon_seeds pitcher_pod pumpkin_seeds torchflower_seeds wheat_seeds",
    ),
    ("minecraft:parrot_poisonous_food", "cookie"),
    (
        "minecraft:pickaxes",
        "copper_pickaxe diamond_pickaxe golden_pickaxe iron_pickaxe netherite_pickaxe stone_pickaxe wooden_pickaxe",
    ),
    ("minecraft:pig_food", "beetroot carrot potato"),
    ("minecraft:piglin_food", "cooked_porkchop porkchop"),
    (
        "minecraft:piglin_loved",
        "bell clock deepslate_gold_ore enchanted_golden_apple gilded_blackstone glistering_melon_slice gold_block gold_ingot gold_ore golden_apple golden_axe golden_boots golden_carrot golden_chestplate golden_dandelion golden_helmet golden_hoe golden_horse_armor golden_leggings golden_nautilus_armor golden_pickaxe golden_shovel golden_spear golden_sword light_weighted_pressure_plate nether_gold_ore raw_gold raw_gold_block",
    ),
    ("minecraft:piglin_preferred_weapons", "crossbow golden_spear"),
    (
        "minecraft:piglin_repellents",
        "soul_campfire soul_lantern soul_torch",
    ),
    (
        "minecraft:piglin_safe_armor",
        "golden_boots golden_chestplate golden_helmet golden_leggings",
    ),
    ("minecraft:pillager_preferred_weapons", "crossbow"),
    (
        "minecraft:planks",
        "acacia_planks bamboo_planks birch_planks cherry_planks crimson_planks dark_oak_planks jungle_planks mangrove_planks oak_planks pale_oak_planks poplar_planks spruce_planks warped_planks",
    ),
    (
        "minecraft:poplar_logs",
        "poplar_log poplar_wood stripped_poplar_log stripped_poplar_wood",
    ),
    ("minecraft:rabbit_food", "carrot dandelion golden_carrot"),
    (
        "minecraft:rails",
        "activator_rail detector_rail powered_rail rail",
    ),
    ("minecraft:redstone_ores", "deepslate_redstone_ore redstone_ore"),
    ("minecraft:repairs_chain_armor", "iron_ingot"),
    ("minecraft:repairs_copper_armor", "copper_ingot"),
    ("minecraft:repairs_diamond_armor", "diamond"),
    ("minecraft:repairs_gold_armor", "gold_ingot"),
    ("minecraft:repairs_iron_armor", "iron_ingot"),
    ("minecraft:repairs_leather_armor", "leather"),
    ("minecraft:repairs_netherite_armor", "netherite_ingot"),
    ("minecraft:repairs_turtle_helmet", "turtle_scute"),
    ("minecraft:repairs_wolf_armor", "armadillo_scute"),
    ("minecraft:sand", "red_sand sand suspicious_sand"),
    (
        "minecraft:saplings",
        "acacia_sapling azalea birch_sapling cherry_sapling dark_oak_sapling flowering_azalea jungle_sapling mangrove_propagule oak_sapling pale_oak_sapling poplar_sapling spruce_sapling",
    ),
    ("minecraft:shearable_from_copper_golem", "poppy"),
    ("minecraft:sheep_food", "wheat"),
    (
        "minecraft:shovels",
        "copper_shovel diamond_shovel golden_shovel iron_shovel netherite_shovel stone_shovel wooden_shovel",
    ),
    (
        "minecraft:shulker_boxes",
        "black_shulker_box blue_shulker_box brown_shulker_box cyan_shulker_box gray_shulker_box green_shulker_box light_blue_shulker_box light_gray_shulker_box lime_shulker_box magenta_shulker_box orange_shulker_box pink_shulker_box purple_shulker_box red_shulker_box shulker_box white_shulker_box yellow_shulker_box",
    ),
    (
        "minecraft:signs",
        "acacia_sign bamboo_sign birch_sign cherry_sign crimson_sign dark_oak_sign jungle_sign mangrove_sign oak_sign pale_oak_sign poplar_sign spruce_sign warped_sign",
    ),
    ("minecraft:skeleton_preferred_weapons", "bow"),
    (
        "minecraft:skulls",
        "creeper_head dragon_head piglin_head player_head skeleton_skull wither_skeleton_skull zombie_head",
    ),
    (
        "minecraft:slabs",
        "acacia_slab andesite_slab bamboo_mosaic_slab bamboo_slab birch_slab black_concrete_slab black_wool_slab blackstone_slab blue_concrete_slab blue_wool_slab brick_slab brown_concrete_slab brown_wool_slab cherry_slab cinnabar_brick_slab cinnabar_slab cobbled_deepslate_slab cobblestone_slab crimson_slab cut_copper_slab cut_red_sandstone_slab cut_sandstone_slab cyan_concrete_slab cyan_wool_slab dark_oak_slab dark_prismarine_slab deepslate_brick_slab deepslate_tile_slab diorite_slab end_stone_brick_slab exposed_cut_copper_slab granite_slab gray_concrete_slab gray_wool_slab green_concrete_slab green_wool_slab jungle_slab light_blue_concrete_slab light_blue_wool_slab light_gray_concrete_slab light_gray_wool_slab lime_concrete_slab lime_wool_slab magenta_concrete_slab magenta_wool_slab mangrove_slab mossy_cobblestone_slab mossy_stone_brick_slab mud_brick_slab nether_brick_slab oak_slab orange_concrete_slab orange_wool_slab oxidized_cut_copper_slab pale_oak_slab petrified_oak_slab pink_concrete_slab pink_wool_slab polished_andesite_slab polished_blackstone_brick_slab polished_blackstone_slab polished_cinnabar_slab polished_deepslate_slab polished_diorite_slab polished_granite_slab polished_sulfur_slab polished_tuff_slab prismarine_brick_slab prismarine_slab purple_concrete_slab purple_wool_slab purpur_slab quartz_slab red_concrete_slab red_nether_brick_slab red_sandstone_slab red_wool_slab resin_brick_slab sandstone_slab smooth_quartz_slab smooth_red_sandstone_slab smooth_sandstone_slab smooth_stone_slab spruce_slab stone_brick_slab stone_slab sulfur_brick_slab sulfur_slab tuff_brick_slab tuff_slab warped_slab waxed_cut_copper_slab waxed_exposed_cut_copper_slab waxed_oxidized_cut_copper_slab waxed_weathered_cut_copper_slab weathered_cut_copper_slab white_concrete_slab white_wool_slab yellow_concrete_slab yellow_wool_slab",
    ),
    (
        "minecraft:small_flowers",
        "allium azure_bluet blue_orchid closed_eyeblossom cornflower dandelion golden_dandelion lily_of_the_valley open_eyeblossom orange_tulip oxeye_daisy pink_tulip poppy red_tulip torchflower white_tulip wither_rose",
    ),
    ("minecraft:smelts_to_glass", "red_sand sand"),
    ("minecraft:sniffer_food", "torchflower_seeds"),
    ("minecraft:soul_fire_base_blocks", "soul_sand soul_soil"),
    (
        "minecraft:spears",
        "copper_spear diamond_spear golden_spear iron_spear netherite_spear stone_spear wooden_spear",
    ),
    (
        "minecraft:spruce_logs",
        "spruce_log spruce_wood stripped_spruce_log stripped_spruce_wood",
    ),
    (
        "minecraft:stairs",
        "acacia_stairs andesite_stairs bamboo_mosaic_stairs bamboo_stairs birch_stairs black_concrete_stairs black_wool_stairs blackstone_stairs blue_concrete_stairs blue_wool_stairs brick_stairs brown_concrete_stairs brown_wool_stairs cherry_stairs cinnabar_brick_stairs cinnabar_stairs cobbled_deepslate_stairs cobblestone_stairs crimson_stairs cut_copper_stairs cyan_concrete_stairs cyan_wool_stairs dark_oak_stairs dark_prismarine_stairs deepslate_brick_stairs deepslate_tile_stairs diorite_stairs end_stone_brick_stairs exposed_cut_copper_stairs granite_stairs gray_concrete_stairs gray_wool_stairs green_concrete_stairs green_wool_stairs jungle_stairs light_blue_concrete_stairs light_blue_wool_stairs light_gray_concrete_stairs light_gray_wool_stairs lime_concrete_stairs lime_wool_stairs magenta_concrete_stairs magenta_wool_stairs mangrove_stairs mossy_cobblestone_stairs mossy_stone_brick_stairs mud_brick_stairs nether_brick_stairs oak_stairs orange_concrete_stairs orange_wool_stairs oxidized_cut_copper_stairs pale_oak_stairs pink_concrete_stairs pink_wool_stairs polished_andesite_stairs polished_blackstone_brick_stairs polished_blackstone_stairs polished_cinnabar_stairs polished_deepslate_stairs polished_diorite_stairs polished_granite_stairs polished_sulfur_stairs polished_tuff_stairs poplar_stairs prismarine_brick_stairs prismarine_stairs purple_concrete_stairs purple_wool_stairs purpur_stairs quartz_stairs red_concrete_stairs red_nether_brick_stairs red_sandstone_stairs red_wool_stairs resin_brick_stairs sandstone_stairs smooth_quartz_stairs smooth_red_sandstone_stairs smooth_sandstone_stairs spruce_stairs stone_brick_stairs stone_stairs sulfur_brick_stairs sulfur_stairs tuff_brick_stairs tuff_stairs warped_stairs waxed_cut_copper_stairs waxed_exposed_cut_copper_stairs waxed_oxidized_cut_copper_stairs waxed_weathered_cut_copper_stairs weathered_cut_copper_stairs white_concrete_stairs white_wool_stairs yellow_concrete_stairs yellow_wool_stairs",
    ),
    (
        "minecraft:stone_bricks",
        "chiseled_stone_bricks cracked_stone_bricks mossy_stone_bricks stone_bricks",
    ),
    (
        "minecraft:stone_buttons",
        "polished_blackstone_button stone_button",
    ),
    (
        "minecraft:stone_crafting_materials",
        "blackstone cobbled_deepslate cobblestone",
    ),
    (
        "minecraft:stone_tool_materials",
        "blackstone cobbled_deepslate cobblestone",
    ),
    ("minecraft:strider_food", "warped_fungus"),
    (
        "minecraft:strider_tempt_items",
        "warped_fungus warped_fungus_on_a_stick",
    ),
    (
        "minecraft:sulfur_cube_archetype/bouncy",
        "acacia_log acacia_planks acacia_wood bamboo_block bamboo_mosaic bamboo_planks birch_log birch_planks birch_wood cherry_log cherry_planks cherry_wood crimson_hyphae crimson_planks crimson_stem dark_oak_log dark_oak_planks dark_oak_wood jungle_log jungle_planks jungle_wood mangrove_log mangrove_planks mangrove_wood oak_log oak_planks oak_wood pale_oak_log pale_oak_planks pale_oak_wood poplar_log poplar_planks poplar_wood spruce_log spruce_planks spruce_wood stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem warped_hyphae warped_planks warped_stem",
    ),
    ("minecraft:sulfur_cube_archetype/explosive", "tnt"),
    (
        "minecraft:sulfur_cube_archetype/fast_flat",
        "brain_coral_block bubble_coral_block carved_pumpkin chiseled_resin_bricks dead_brain_coral_block dead_bubble_coral_block dead_fire_coral_block dead_horn_coral_block dead_tube_coral_block dried_kelp_block fire_coral_block hay_block horn_coral_block jack_o_lantern melon moss_block ochre_froglight pale_moss_block pearlescent_froglight pumpkin resin_block resin_bricks sponge tube_coral_block verdant_froglight wet_sponge",
    ),
    (
        "minecraft:sulfur_cube_archetype/fast_sliding",
        "blue_ice packed_ice snow_block",
    ),
    (
        "minecraft:sulfur_cube_archetype/high_resistance",
        "soul_sand soul_soil",
    ),
    ("minecraft:sulfur_cube_archetype/hot", "magma_block"),
    (
        "minecraft:sulfur_cube_archetype/light",
        "black_wool blue_wool brown_wool cyan_wool gray_wool green_wool light_blue_wool light_gray_wool lime_wool magenta_wool orange_wool pink_wool purple_wool red_wool white_wool yellow_wool",
    ),
    (
        "minecraft:sulfur_cube_archetype/regular",
        "black_concrete_powder blue_concrete_powder bone_block brown_concrete_powder clay coal_block coarse_dirt cyan_concrete_powder dirt grass_block gray_concrete_powder green_concrete_powder light_blue_concrete_powder light_gray_concrete_powder lime_concrete_powder magenta_concrete_powder mud muddy_mangrove_roots orange_concrete_powder packed_mud pink_concrete_powder podzol purple_concrete_powder red_concrete_powder rooted_dirt white_concrete_powder yellow_concrete_powder",
    ),
    (
        "minecraft:sulfur_cube_archetype/slow_bouncy",
        "amethyst_block andesite basalt black_concrete black_glazed_terracotta black_terracotta blackstone blue_concrete blue_glazed_terracotta blue_terracotta bricks brown_concrete brown_glazed_terracotta brown_terracotta calcite chiseled_cinnabar chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_bricks coal_ore cobbled_deepslate cobblestone cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crimson_nylium crying_obsidian cut_red_sandstone cut_sandstone cyan_concrete cyan_glazed_terracotta cyan_terracotta dark_prismarine deepslate deepslate_bricks deepslate_coal_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tiles diamond_block diamond_ore diorite dripstone_block emerald_block emerald_ore end_stone end_stone_bricks gilded_blackstone glowstone granite gray_concrete gray_glazed_terracotta gray_terracotta green_concrete green_glazed_terracotta green_terracotta lapis_block lapis_ore light_blue_concrete light_blue_glazed_terracotta light_blue_terracotta light_gray_concrete light_gray_glazed_terracotta light_gray_terracotta lime_concrete lime_glazed_terracotta lime_terracotta magenta_concrete magenta_glazed_terracotta magenta_terracotta mossy_cobblestone mossy_stone_bricks mud_bricks nether_bricks nether_quartz_ore netherrack observer obsidian orange_concrete orange_glazed_terracotta orange_terracotta pink_concrete pink_glazed_terracotta pink_terracotta polished_andesite polished_basalt polished_blackstone polished_blackstone_bricks polished_cinnabar polished_deepslate polished_diorite polished_granite polished_sulfur polished_tuff prismarine prismarine_bricks purple_concrete purple_glazed_terracotta purple_terracotta purpur_block purpur_pillar quartz_block quartz_bricks quartz_pillar red_concrete red_glazed_terracotta red_nether_bricks red_sandstone red_terracotta redstone_lamp redstone_ore sandstone sea_lantern smooth_basalt smooth_quartz smooth_red_sandstone smooth_sandstone smooth_stone stone stone_bricks sulfur sulfur_bricks terracotta tuff tuff_bricks warped_nylium white_concrete white_glazed_terracotta white_terracotta yellow_concrete yellow_glazed_terracotta yellow_terracotta",
    ),
    (
        "minecraft:sulfur_cube_archetype/slow_flat",
        "ancient_debris chiseled_copper copper_block copper_bulb copper_ore cut_copper deepslate_copper_ore deepslate_gold_ore deepslate_iron_ore exposed_chiseled_copper exposed_copper exposed_copper_bulb exposed_cut_copper gold_block gold_ore iron_block iron_ore nether_gold_ore netherite_block oxidized_chiseled_copper oxidized_copper oxidized_copper_bulb oxidized_cut_copper raw_copper_block raw_gold_block raw_iron_block waxed_chiseled_copper waxed_copper_block waxed_copper_bulb waxed_cut_copper waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bulb waxed_exposed_cut_copper waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bulb waxed_oxidized_cut_copper waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bulb waxed_weathered_cut_copper weathered_chiseled_copper weathered_copper weathered_copper_bulb weathered_cut_copper",
    ),
    (
        "minecraft:sulfur_cube_archetype/slow_sliding",
        "brown_mushroom_block mushroom_stem mycelium nether_wart_block red_mushroom_block shroomlight warped_wart_block",
    ),
    ("minecraft:sulfur_cube_archetype/sticky", "honeycomb_block"),
    ("minecraft:sulfur_cube_food", "slime_ball"),
    (
        "minecraft:sulfur_cube_swallowable",
        "acacia_log acacia_planks acacia_wood amethyst_block ancient_debris andesite bamboo_block bamboo_mosaic bamboo_planks basalt birch_log birch_planks birch_wood black_concrete black_concrete_powder black_glazed_terracotta black_terracotta black_wool blackstone blue_concrete blue_concrete_powder blue_glazed_terracotta blue_ice blue_terracotta blue_wool bone_block brain_coral_block bricks brown_concrete brown_concrete_powder brown_glazed_terracotta brown_mushroom_block brown_terracotta brown_wool bubble_coral_block calcite carved_pumpkin cherry_log cherry_planks cherry_wood chiseled_cinnabar chiseled_copper chiseled_deepslate chiseled_nether_bricks chiseled_polished_blackstone chiseled_quartz_block chiseled_red_sandstone chiseled_resin_bricks chiseled_sandstone chiseled_stone_bricks chiseled_sulfur chiseled_tuff chiseled_tuff_bricks cinnabar cinnabar_bricks clay coal_block coal_ore coarse_dirt cobbled_deepslate cobblestone copper_block copper_bulb copper_ore cracked_deepslate_bricks cracked_deepslate_tiles cracked_nether_bricks cracked_polished_blackstone_bricks cracked_stone_bricks crimson_hyphae crimson_nylium crimson_planks crimson_stem crying_obsidian cut_copper cut_red_sandstone cut_sandstone cyan_concrete cyan_concrete_powder cyan_glazed_terracotta cyan_terracotta cyan_wool dark_oak_log dark_oak_planks dark_oak_wood dark_prismarine dead_brain_coral_block dead_bubble_coral_block dead_fire_coral_block dead_horn_coral_block dead_tube_coral_block deepslate deepslate_bricks deepslate_coal_ore deepslate_copper_ore deepslate_diamond_ore deepslate_emerald_ore deepslate_gold_ore deepslate_iron_ore deepslate_lapis_ore deepslate_redstone_ore deepslate_tiles diamond_block diamond_ore diorite dirt dried_kelp_block dripstone_block emerald_block emerald_ore end_stone end_stone_bricks exposed_chiseled_copper exposed_copper exposed_copper_bulb exposed_cut_copper fire_coral_block gilded_blackstone glowstone gold_block gold_ore granite grass_block gray_concrete gray_concrete_powder gray_glazed_terracotta gray_terracotta gray_wool green_concrete green_concrete_powder green_glazed_terracotta green_terracotta green_wool hay_block honeycomb_block horn_coral_block iron_block iron_ore jack_o_lantern jungle_log jungle_planks jungle_wood lapis_block lapis_ore light_blue_concrete light_blue_concrete_powder light_blue_glazed_terracotta light_blue_terracotta light_blue_wool light_gray_concrete light_gray_concrete_powder light_gray_glazed_terracotta light_gray_terracotta light_gray_wool lime_concrete lime_concrete_powder lime_glazed_terracotta lime_terracotta lime_wool magenta_concrete magenta_concrete_powder magenta_glazed_terracotta magenta_terracotta magenta_wool magma_block mangrove_log mangrove_planks mangrove_wood melon moss_block mossy_cobblestone mossy_stone_bricks mud mud_bricks muddy_mangrove_roots mushroom_stem mycelium nether_bricks nether_gold_ore nether_quartz_ore nether_wart_block netherite_block netherrack oak_log oak_planks oak_wood observer obsidian ochre_froglight orange_concrete orange_concrete_powder orange_glazed_terracotta orange_terracotta orange_wool oxidized_chiseled_copper oxidized_copper oxidized_copper_bulb oxidized_cut_copper packed_ice packed_mud pale_moss_block pale_oak_log pale_oak_planks pale_oak_wood pearlescent_froglight pink_concrete pink_concrete_powder pink_glazed_terracotta pink_terracotta pink_wool podzol polished_andesite polished_basalt polished_blackstone polished_blackstone_bricks polished_cinnabar polished_deepslate polished_diorite polished_granite polished_sulfur polished_tuff poplar_log poplar_planks poplar_wood prismarine prismarine_bricks pumpkin purple_concrete purple_concrete_powder purple_glazed_terracotta purple_terracotta purple_wool purpur_block purpur_pillar quartz_block quartz_bricks quartz_pillar raw_copper_block raw_gold_block raw_iron_block red_concrete red_concrete_powder red_glazed_terracotta red_mushroom_block red_nether_bricks red_sandstone red_terracotta red_wool redstone_lamp redstone_ore resin_block resin_bricks rooted_dirt sandstone sea_lantern shroomlight smooth_basalt smooth_quartz smooth_red_sandstone smooth_sandstone smooth_stone snow_block soul_sand soul_soil sponge spruce_log spruce_planks spruce_wood stone stone_bricks stripped_acacia_log stripped_acacia_wood stripped_bamboo_block stripped_birch_log stripped_birch_wood stripped_cherry_log stripped_cherry_wood stripped_crimson_hyphae stripped_crimson_stem stripped_dark_oak_log stripped_dark_oak_wood stripped_jungle_log stripped_jungle_wood stripped_mangrove_log stripped_mangrove_wood stripped_oak_log stripped_oak_wood stripped_pale_oak_log stripped_pale_oak_wood stripped_poplar_log stripped_poplar_wood stripped_spruce_log stripped_spruce_wood stripped_warped_hyphae stripped_warped_stem sulfur sulfur_bricks terracotta tnt tube_coral_block tuff tuff_bricks verdant_froglight warped_hyphae warped_nylium warped_planks warped_stem warped_wart_block waxed_chiseled_copper waxed_copper_block waxed_copper_bulb waxed_cut_copper waxed_exposed_chiseled_copper waxed_exposed_copper waxed_exposed_copper_bulb waxed_exposed_cut_copper waxed_oxidized_chiseled_copper waxed_oxidized_copper waxed_oxidized_copper_bulb waxed_oxidized_cut_copper waxed_weathered_chiseled_copper waxed_weathered_copper waxed_weathered_copper_bulb waxed_weathered_cut_copper weathered_chiseled_copper weathered_copper weathered_copper_bulb weathered_cut_copper wet_sponge white_concrete white_concrete_powder white_glazed_terracotta white_terracotta white_wool yellow_concrete yellow_concrete_powder yellow_glazed_terracotta yellow_terracotta yellow_wool",
    ),
    (
        "minecraft:swords",
        "copper_sword diamond_sword golden_sword iron_sword netherite_sword stone_sword wooden_sword",
    ),
    (
        "minecraft:terracotta",
        "black_terracotta blue_terracotta brown_terracotta cyan_terracotta gray_terracotta green_terracotta light_blue_terracotta light_gray_terracotta lime_terracotta magenta_terracotta orange_terracotta pink_terracotta purple_terracotta red_terracotta terracotta white_terracotta yellow_terracotta",
    ),
    (
        "minecraft:trapdoors",
        "acacia_trapdoor bamboo_trapdoor birch_trapdoor cherry_trapdoor copper_trapdoor crimson_trapdoor dark_oak_trapdoor exposed_copper_trapdoor iron_trapdoor jungle_trapdoor mangrove_trapdoor oak_trapdoor oxidized_copper_trapdoor pale_oak_trapdoor poplar_trapdoor spruce_trapdoor warped_trapdoor waxed_copper_trapdoor waxed_exposed_copper_trapdoor waxed_oxidized_copper_trapdoor waxed_weathered_copper_trapdoor weathered_copper_trapdoor",
    ),
    (
        "minecraft:trim_materials",
        "amethyst_shard copper_ingot diamond emerald gold_ingot iron_ingot lapis_lazuli netherite_ingot quartz redstone resin_brick",
    ),
    (
        "minecraft:trimmable_armor",
        "chainmail_boots chainmail_chestplate chainmail_helmet chainmail_leggings copper_boots copper_chestplate copper_helmet copper_leggings diamond_boots diamond_chestplate diamond_helmet diamond_leggings golden_boots golden_chestplate golden_helmet golden_leggings iron_boots iron_chestplate iron_helmet iron_leggings leather_boots leather_chestplate leather_helmet leather_leggings netherite_boots netherite_chestplate netherite_helmet netherite_leggings turtle_helmet",
    ),
    ("minecraft:turtle_food", "seagrass"),
    (
        "minecraft:villager_picks_up",
        "beetroot beetroot_seeds bread carrot pitcher_pod potato torchflower_seeds wheat wheat_seeds",
    ),
    (
        "minecraft:villager_plantable_seeds",
        "beetroot_seeds carrot pitcher_pod potato torchflower_seeds wheat_seeds",
    ),
    (
        "minecraft:walls",
        "andesite_wall blackstone_wall brick_wall cinnabar_brick_wall cinnabar_wall cobbled_deepslate_wall cobblestone_wall deepslate_brick_wall deepslate_tile_wall diorite_wall end_stone_brick_wall granite_wall mossy_cobblestone_wall mossy_stone_brick_wall mud_brick_wall nether_brick_wall polished_blackstone_brick_wall polished_blackstone_wall polished_cinnabar_wall polished_deepslate_wall polished_sulfur_wall polished_tuff_wall prismarine_wall red_nether_brick_wall red_sandstone_wall resin_brick_wall sandstone_wall stone_brick_wall sulfur_brick_wall sulfur_wall tuff_brick_wall tuff_wall",
    ),
    (
        "minecraft:warped_stems",
        "stripped_warped_hyphae stripped_warped_stem warped_hyphae warped_stem",
    ),
    ("minecraft:wart_blocks", "nether_wart_block warped_wart_block"),
    ("minecraft:wither_skeleton_disliked_weapons", "bow crossbow"),
    (
        "minecraft:wolf_collar_dyes",
        "black_dye blue_dye brown_dye cyan_dye gray_dye green_dye light_blue_dye light_gray_dye lime_dye magenta_dye orange_dye pink_dye purple_dye red_dye white_dye yellow_dye",
    ),
    (
        "minecraft:wolf_food",
        "beef chicken cod cooked_beef cooked_chicken cooked_cod cooked_mutton cooked_porkchop cooked_rabbit cooked_salmon mutton porkchop pufferfish rabbit rabbit_stew rotten_flesh salmon tropical_fish",
    ),
    (
        "minecraft:wooden_buttons",
        "acacia_button bamboo_button birch_button cherry_button crimson_button dark_oak_button jungle_button mangrove_button oak_button pale_oak_button poplar_button spruce_button warped_button",
    ),
    (
        "minecraft:wooden_doors",
        "acacia_door bamboo_door birch_door cherry_door crimson_door dark_oak_door jungle_door mangrove_door oak_door pale_oak_door poplar_door spruce_door warped_door",
    ),
    (
        "minecraft:wooden_fences",
        "acacia_fence bamboo_fence birch_fence cherry_fence crimson_fence dark_oak_fence jungle_fence mangrove_fence oak_fence pale_oak_fence poplar_fence spruce_fence warped_fence",
    ),
    (
        "minecraft:wooden_pressure_plates",
        "acacia_pressure_plate bamboo_pressure_plate birch_pressure_plate cherry_pressure_plate crimson_pressure_plate dark_oak_pressure_plate jungle_pressure_plate mangrove_pressure_plate oak_pressure_plate pale_oak_pressure_plate poplar_pressure_plate spruce_pressure_plate warped_pressure_plate",
    ),
    (
        "minecraft:wooden_shelves",
        "acacia_shelf bamboo_shelf birch_shelf cherry_shelf crimson_shelf dark_oak_shelf jungle_shelf mangrove_shelf oak_shelf pale_oak_shelf poplar_shelf spruce_shelf warped_shelf",
    ),
    (
        "minecraft:wooden_slabs",
        "acacia_slab bamboo_slab birch_slab cherry_slab crimson_slab dark_oak_slab jungle_slab mangrove_slab oak_slab pale_oak_slab spruce_slab warped_slab",
    ),
    (
        "minecraft:wooden_stairs",
        "acacia_stairs bamboo_stairs birch_stairs cherry_stairs crimson_stairs dark_oak_stairs jungle_stairs mangrove_stairs oak_stairs pale_oak_stairs poplar_stairs spruce_stairs warped_stairs",
    ),
    (
        "minecraft:wooden_tool_materials",
        "acacia_planks bamboo_planks birch_planks cherry_planks crimson_planks dark_oak_planks jungle_planks mangrove_planks oak_planks pale_oak_planks poplar_planks spruce_planks warped_planks",
    ),
    (
        "minecraft:wooden_trapdoors",
        "acacia_trapdoor bamboo_trapdoor birch_trapdoor cherry_trapdoor crimson_trapdoor dark_oak_trapdoor jungle_trapdoor mangrove_trapdoor oak_trapdoor pale_oak_trapdoor poplar_trapdoor spruce_trapdoor warped_trapdoor",
    ),
    (
        "minecraft:wool",
        "black_wool blue_wool brown_wool cyan_wool gray_wool green_wool light_blue_wool light_gray_wool lime_wool magenta_wool orange_wool pink_wool purple_wool red_wool white_wool yellow_wool",
    ),
    (
        "minecraft:wool_carpets",
        "black_carpet blue_carpet brown_carpet cyan_carpet gray_carpet green_carpet light_blue_carpet light_gray_carpet lime_carpet magenta_carpet orange_carpet pink_carpet purple_carpet red_carpet white_carpet yellow_carpet",
    ),
    (
        "minecraft:wool_slabs",
        "black_wool_slab blue_wool_slab brown_wool_slab cyan_wool_slab gray_wool_slab green_wool_slab light_blue_wool_slab light_gray_wool_slab lime_wool_slab magenta_wool_slab orange_wool_slab pink_wool_slab purple_wool_slab red_wool_slab white_wool_slab yellow_wool_slab",
    ),
    (
        "minecraft:wool_stairs",
        "black_wool_stairs blue_wool_stairs brown_wool_stairs cyan_wool_stairs gray_wool_stairs green_wool_stairs light_blue_wool_stairs light_gray_wool_stairs lime_wool_stairs magenta_wool_stairs orange_wool_stairs pink_wool_stairs purple_wool_stairs red_wool_stairs white_wool_stairs yellow_wool_stairs",
    ),
    ("minecraft:zombie_horse_food", "red_mushroom"),
];
