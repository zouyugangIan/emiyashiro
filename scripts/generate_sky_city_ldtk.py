#!/usr/bin/env python3
"""Generate the production LDtk 1.5.3 project for Windheart Sky City.

The file is deliberately generated from compact authored primitives so the long
level remains reviewable and reproducible while LDtk stays the source of truth.
"""

from __future__ import annotations

import argparse
import json
import uuid
from pathlib import Path

GRID = 32
WIDTH = 384
HEIGHT = 48
LEVEL_UID = 1

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "assets" / "levels" / "sky_city_of_winds.ldtk"
TILESET_UID = 400
TILESET_PATH = "../images/levels/sky_city_tiles.png"
IID_NAMESPACE = uuid.UUID("783348c8-42ce-5c64-96e5-d55497e56f2d")


def iid(stable_key: str) -> str:
    """Return an editor-safe IID that does not churn on regeneration."""
    return str(uuid.uuid5(IID_NAMESPACE, stable_key))


def field_def(identifier: str, uid: int, display_type: str, storage_type: str) -> dict:
    return {
        "identifier": identifier,
        "doc": None,
        "__type": display_type,
        "uid": uid,
        "type": storage_type,
        "isArray": False,
        "canBeNull": False,
        "arrayMinLength": None,
        "arrayMaxLength": None,
        "editorDisplayMode": "NameAndValue",
        "editorDisplayScale": 1,
        "editorDisplayPos": "Above",
        "editorLinkStyle": "StraightArrow",
        "editorDisplayColor": None,
        "editorAlwaysShow": False,
        "editorShowInWorld": True,
        "editorCutLongValues": True,
        "editorTextSuffix": None,
        "editorTextPrefix": None,
        "useForSmartColor": False,
        "exportToToc": False,
        "searchable": False,
        "min": None,
        "max": None,
        "regex": None,
        "acceptFileTypes": None,
        "defaultOverride": None,
        "textLanguageMode": None,
        "symmetricalRef": False,
        "autoChainRef": True,
        "allowOutOfLevelRef": True,
        "allowedRefs": "Any",
        "allowedRefsEntityUid": None,
        "allowedRefTags": [],
        "tilesetUid": None,
    }


def entity_def(
    identifier: str,
    uid: int,
    color: str,
    fields: list[dict] | None = None,
    width: int = GRID,
    height: int = GRID,
    resizable_x: bool = False,
    resizable_y: bool = False,
    max_count: int = 0,
) -> dict:
    return {
        "identifier": identifier,
        "uid": uid,
        "tags": [],
        "exportToToc": False,
        "allowOutOfBounds": False,
        "doc": None,
        "width": width,
        "height": height,
        "resizableX": resizable_x,
        "resizableY": resizable_y,
        "minWidth": None,
        "maxWidth": None,
        "minHeight": None,
        "maxHeight": None,
        "keepAspectRatio": False,
        "tileOpacity": 1,
        "fillOpacity": 0.42,
        "lineOpacity": 1,
        "hollow": False,
        "color": color,
        "renderMode": "Rectangle",
        "showName": True,
        "tilesetId": None,
        "tileRenderMode": "Cover",
        "tileRect": None,
        "uiTileRect": None,
        "nineSliceBorders": [],
        "maxCount": max_count,
        "limitScope": "PerLevel",
        "limitBehavior": "MoveLastOne",
        "pivotX": 0,
        "pivotY": 0,
        "fieldDefs": fields or [],
    }


def layer_def(identifier: str, uid: int, layer_type: str) -> dict:
    return {
        "__type": layer_type,
        "identifier": identifier,
        "type": layer_type,
        "uid": uid,
        "doc": None,
        "uiColor": None,
        "gridSize": GRID,
        "guideGridWid": 0,
        "guideGridHei": 0,
        "displayOpacity": 1,
        "inactiveOpacity": 0.55,
        "hideInList": False,
        "hideFieldsWhenInactive": layer_type == "Entities",
        "canSelectWhenInactive": True,
        "renderInWorldView": True,
        "pxOffsetX": 0,
        "pxOffsetY": 0,
        "parallaxFactorX": 0,
        "parallaxFactorY": 0,
        "parallaxScaling": True,
        "requiredTags": [],
        "excludedTags": [],
        "autoTilesKilledByOtherLayerUid": None,
        "uiFilterTags": [],
        "useAsyncRender": False,
        "intGridValues": [],
        "intGridValuesGroups": [],
        "autoRuleGroups": [],
        "autoSourceLayerDefUid": None,
        "tilesetDefUid": None,
        "tilePivotX": 0,
        "tilePivotY": 0,
        "biomeFieldUid": None,
    }


def field_instance(identifier: str, def_uid: int, display_type: str, value) -> dict:
    if value is None:
        real = []
    elif isinstance(value, bool):
        real = [{"id": "V_Bool", "params": [value]}]
    elif isinstance(value, int):
        real = [{"id": "V_Int", "params": [value]}]
    elif isinstance(value, float):
        real = [{"id": "V_Float", "params": [value]}]
    else:
        real = [{"id": "V_String", "params": [value]}]
    return {
        "__identifier": identifier,
        "__type": display_type,
        "__value": value,
        "__tile": None,
        "defUid": def_uid,
        "realEditorValues": real,
    }


ENTITY_UIDS = {
    "PlayerStart": 200,
    "Checkpoint": 201,
    "EnemySpawn": 202,
    "CombatGate": 203,
    "Goal": 204,
    "Backdrop": 205,
}


def entity_instance(
    identifier: str,
    gx: int,
    gy: int,
    color: str,
    fields: list[dict] | None = None,
    width: int = GRID,
    height: int = GRID,
) -> dict:
    top_y = HEIGHT - 1 - gy
    return {
        "__identifier": identifier,
        "__grid": [gx, top_y],
        "__pivot": [0, 0],
        "__tags": [],
        "__tile": None,
        "__smartColor": color,
        "iid": iid(f"entity:{identifier}:{gx}:{gy}"),
        "width": width,
        "height": height,
        "defUid": ENTITY_UIDS[identifier],
        "px": [gx * GRID, top_y * GRID],
        "fieldInstances": fields or [],
        "__worldX": gx * GRID,
        "__worldY": top_y * GRID,
    }


def layer_instance(identifier: str, layer_type: str, def_uid: int) -> dict:
    return {
        "__identifier": identifier,
        "__type": layer_type,
        "__cWid": WIDTH,
        "__cHei": HEIGHT,
        "__gridSize": GRID,
        "__opacity": 1,
        "__pxTotalOffsetX": 0,
        "__pxTotalOffsetY": 0,
        "__tilesetDefUid": None,
        "__tilesetRelPath": None,
        "iid": iid(f"layer:{def_uid}:{identifier}"),
        "levelId": LEVEL_UID,
        "layerDefUid": def_uid,
        "pxOffsetX": 0,
        "pxOffsetY": 0,
        "visible": True,
        "optionalRules": [],
        "intGridCsv": [],
        "autoLayerTiles": [],
        "seed": 20260714 + def_uid,
        "overrideTilesetUid": None,
        "gridTiles": [],
        "entityInstances": [],
    }


def build_collision() -> list[list[int]]:
    cells = [[0 for _ in range(WIDTH)] for _ in range(HEIGHT)]

    def platform(left: int, right: int, top: int, depth: int = 3, material: int = 1) -> None:
        for y in range(max(0, top - depth), top):
            for x in range(max(0, left), min(WIDTH, right + 1)):
                cells[y][x] = material

    def hazard(x: int, y: int) -> None:
        if 0 <= x < WIDTH and 0 <= y < HEIGHT and cells[y][x] == 0:
            cells[y][x] = 3

    def wind_column(x: int, bottom: int, top: int) -> None:
        for y in range(bottom, top + 1):
            if 0 <= x < WIDTH and 0 <= y < HEIGHT and cells[y][x] == 0:
                cells[y][x] = 4

            if 0 <= x + 1 < WIDTH and cells[y][x + 1] == 0:
                cells[y][x + 1] = 4


    # Twelve connected floating districts. Mandatory jumps are 3 cells or less.
    main_islands = [
        (0, 37, 9, 1),
        (41, 64, 10, 1),
        (68, 94, 11, 1),
        (98, 126, 9, 1),
        (130, 158, 12, 1),
        (162, 190, 11, 1),
        (194, 222, 13, 1),
        (226, 254, 10, 2),
        (258, 286, 12, 1),
        (290, 318, 11, 1),
        (322, 350, 13, 2),
        (354, 383, 10, 1),
    ]
    for left, right, top, material in main_islands:
        platform(left, right, top, 4 if material == 1 else 3, material)

    # Layered garden routes and secret balconies.
    for args in [
        (8, 15, 12, 2, 1),
        (18, 27, 15, 2, 1),
        (30, 36, 18, 2, 2),
        (47, 55, 14, 2, 1),
        (58, 63, 17, 2, 2),
        (72, 80, 15, 2, 1),
        (84, 92, 18, 2, 1),
        (102, 110, 13, 2, 1),
        (114, 123, 16, 2, 1),
        (136, 144, 16, 2, 1),
        (147, 155, 19, 2, 2),
        (166, 172, 15, 2, 1),
        (175, 181, 18, 2, 1),
        (184, 189, 21, 2, 2),
        (198, 207, 17, 2, 1),
        (211, 220, 20, 2, 1),
        (230, 238, 14, 2, 2),
        (242, 251, 17, 2, 2),
        (262, 270, 16, 2, 1),
        (274, 283, 19, 2, 1),
        (294, 302, 15, 2, 1),
        (306, 315, 18, 2, 2),
        (326, 334, 17, 2, 2),
        (338, 347, 20, 2, 2),
        (358, 366, 14, 2, 1),
        (370, 380, 17, 2, 1),
    ]:
        platform(*args)

    # The wind-engine shaft is a generous optional vertical traversal route.
    for left, right, top in [
        (168, 172, 24),
        (177, 181, 27),
        (186, 190, 30),
        (195, 200, 33),
        (204, 211, 36),
    ]:
        platform(left, right, top, 2, 2 if top >= 30 else 1)
    wind_column(164, 12, 27)
    wind_column(192, 14, 34)

    # Strong vertical drafts on cloud districts.
    wind_column(223, 11, 19)
    wind_column(253, 11, 19)
    wind_column(319, 12, 22)
    wind_column(351, 11, 21)

    # Readable hazard groupings with safe landing space around each group.
    for x, y in [
        (52, 14), (53, 14),
        (86, 18),
        (117, 16), (118, 16),
        (149, 19),
        (215, 20), (216, 20),
        (247, 17),
        (277, 19), (278, 19),
        (311, 18),
        (343, 20),
        (373, 17), (374, 17),
    ]:
        hazard(x, y)

    return cells


def surface_y(cells: list[list[int]], x: int) -> int:
    solids = [y for y in range(HEIGHT) if cells[y][x] in (1, 2)]
    return max(solids) + 1 if solids else 10


def build_project() -> dict:
    collision = build_collision()

    enemy_kind_uid = 100
    backdrop_kind_uid = 101
    checkpoint_id_uid = 300
    enemy_kind_field_uid = 301
    enemy_arena_field_uid = 302
    enemy_health_field_uid = 303
    gate_arena_field_uid = 304
    backdrop_kind_field_uid = 305
    backdrop_scale_field_uid = 306
    backdrop_depth_field_uid = 307
    enemy_patrol_field_uid = 308

    gameplay = layer_instance("Gameplay", "Entities", 10)
    decor = layer_instance("Decor", "Entities", 11)
    terrain = layer_instance("TerrainTiles", "Tiles", 13)
    terrain["__tilesetDefUid"] = TILESET_UID
    terrain["__tilesetRelPath"] = TILESET_PATH
    collision_layer = layer_instance("Collision", "IntGrid", 12)

    def add_gameplay(
        identifier: str,
        x: int,
        y: int,
        fields=None,
        color="#FFFFFF",
        width: int = GRID,
        height: int = GRID,
    ) -> None:
        gameplay["entityInstances"].append(
            entity_instance(identifier, x, y, color, fields or [], width, height)
        )

    add_gameplay("PlayerStart", 4, surface_y(collision, 4), color="#4FE6FF")

    for checkpoint_id, x in [(0, 5), (1, 112), (2, 205), (3, 302), (4, 360)]:
        add_gameplay(
            "Checkpoint",
            x,
            surface_y(collision, x),
            [field_instance("id", checkpoint_id_uid, "Int", checkpoint_id)],
            "#FFD166",
        )

    enemy_spawns = [
        (23, "Slime", 0, 1.0), (33, "Familiar", 0, 1.0),
        (49, "Slime", 0, 1.0), (60, "Familiar", 0, 1.0),
        (74, "Slime", 0, 1.1), (88, "HeroicSpirit", 0, 0.8),
        (104, "Slime", 0, 1.1), (120, "Familiar", 0, 1.1),
        (135, "Slime", 1, 1.2), (141, "Familiar", 1, 1.2),
        (147, "Slime", 1, 1.3), (153, "HeroicSpirit", 1, 1.35),
        (169, "Slime", 0, 1.25), (179, "Familiar", 0, 1.2),
        (188, "HeroicSpirit", 0, 1.05), (201, "Familiar", 0, 1.25),
        (216, "Slime", 0, 1.35), (234, "Familiar", 0, 1.3),
        (248, "HeroicSpirit", 0, 1.15),
        (263, "Slime", 2, 1.4), (269, "Familiar", 2, 1.35),
        (275, "Slime", 2, 1.5), (281, "HeroicSpirit", 2, 1.55),
        (297, "Familiar", 0, 1.4), (312, "HeroicSpirit", 0, 1.25),
        (328, "Slime", 0, 1.5), (344, "Familiar", 0, 1.45),
        (359, "Slime", 3, 1.7), (364, "Familiar", 3, 1.6),
        (369, "HeroicSpirit", 3, 1.8), (375, "Familiar", 3, 1.8),
        (379, "HeroicSpirit", 3, 2.8),
    ]
    for x, kind, arena, health in enemy_spawns:
        patrol_range = 160.0 if arena else {
            "Slime": 96.0,
            "Familiar": 144.0,
            "HeroicSpirit": 112.0,
        }[kind]
        add_gameplay(
            "EnemySpawn",
            x,
            surface_y(collision, x),
            [
                field_instance("kind", enemy_kind_field_uid, "LocalEnum.EnemyKind", kind),
                field_instance("arena", enemy_arena_field_uid, "Int", arena),
                field_instance("healthMultiplier", enemy_health_field_uid, "Float", health),
                field_instance("patrolRange", enemy_patrol_field_uid, "Float", patrol_range),
            ],
            {"Slime": "#78D689", "Familiar": "#A8A0FF", "HeroicSpirit": "#E85D75"}[kind],
        )

    for arena, left, right in [(1, 130, 158), (2, 258, 286), (3, 354, 382)]:
        for x in (left, right):
            add_gameplay(
                "CombatGate",
                x,
                surface_y(collision, x),
                [field_instance("arena", gate_arena_field_uid, "Int", arena)],
                "#58E6FF",
                height=288,
            )

    add_gameplay("Goal", 379, surface_y(collision, 379), color="#8FFFFF")

    backdrop_specs = [
        ("Cloud", 10, 30, 1.1, 0), ("Island", 24, 25, 0.75, 2),
        ("Tower", 36, 22, 0.62, 3), ("Cloud", 52, 35, 1.4, 0),
        ("Waterfall", 62, 17, 0.8, 4), ("Aqueduct", 78, 26, 0.7, 3),
        ("Island", 94, 32, 0.9, 1), ("Windmill", 108, 24, 0.8, 4),
        ("Cloud", 120, 38, 1.2, 0), ("Temple", 138, 25, 0.72, 3),
        ("Crystal", 151, 20, 0.75, 4), ("Tower", 166, 28, 0.72, 2),
        ("Windmill", 181, 31, 1.05, 3), ("Island", 197, 24, 0.82, 1),
        ("Waterfall", 208, 20, 1.0, 4), ("Tree", 220, 26, 0.9, 3),
        ("Cloud", 236, 36, 1.55, 0), ("Island", 250, 21, 0.68, 3),
        ("Aqueduct", 267, 28, 0.88, 2), ("Temple", 282, 24, 0.80, 3),
        ("Tower", 297, 30, 0.68, 2), ("Crystal", 310, 22, 1.0, 4),
        ("Cloud", 326, 37, 1.4, 0), ("Tree", 339, 25, 1.0, 3),
        ("Island", 352, 30, 0.88, 1), ("Waterfall", 361, 18, 1.05, 4),
        ("Temple", 371, 25, 1.12, 3), ("Crystal", 380, 20, 1.28, 4),
    ]
    for kind, x, y, scale, depth in backdrop_specs:
        decor["entityInstances"].append(
            entity_instance(
                "Backdrop",
                x,
                y,
                "#D8F3FF",
                [
                    field_instance("kind", backdrop_kind_field_uid, "LocalEnum.BackdropKind", kind),
                    field_instance("scale", backdrop_scale_field_uid, "Float", scale),
                    field_instance("depth", backdrop_depth_field_uid, "Int", depth),
                ],
            )
        )

    # LDtk stores IntGrid CSV from top row to bottom row.
    collision_layer["intGridCsv"] = [
        collision[y][x]
        for y in range(HEIGHT - 1, -1, -1)
        for x in range(WIDTH)
    ]

    # A real LDtk Tiles layer lets bevy_ecs_ldtk batch the dense terrain while
    # the IntGrid remains the authoritative gameplay/collision representation.
    for y in range(HEIGHT - 1, -1, -1):
        for x in range(WIDTH):
            value = collision[y][x]
            if value == 0:
                continue
            variant = (x * 17 + y * 29) % 4
            tile_id = {
                1: variant,
                2: 4 + variant % 2,
                3: 6,
                4: 7,
            }[value]
            terrain["gridTiles"].append(
                {
                    "px": [x * GRID, (HEIGHT - 1 - y) * GRID],
                    "src": [tile_id * GRID, 0],
                    "f": 0,
                    "t": tile_id,
                    "d": [x + (HEIGHT - 1 - y) * WIDTH],
                    "a": 1,
                }
            )

    gameplay_def = layer_def("Gameplay", 10, "Entities")
    decor_def = layer_def("Decor", 11, "Entities")
    terrain_def = layer_def("TerrainTiles", 13, "Tiles")
    terrain_def["tilesetDefUid"] = TILESET_UID
    collision_def = layer_def("Collision", 12, "IntGrid")
    collision_def["intGridValues"] = [
        {"value": 1, "identifier": "StoneSolid", "color": "#817B69", "tile": None, "groupUid": 0},
        {"value": 2, "identifier": "CloudSolid", "color": "#E8F1F5", "tile": None, "groupUid": 0},
        {"value": 3, "identifier": "ArcHazard", "color": "#E49B2F", "tile": None, "groupUid": 0},
        {"value": 4, "identifier": "WindLift", "color": "#6FE8FF", "tile": None, "groupUid": 0},
    ]

    enemy_fields = [
        field_def("kind", enemy_kind_field_uid, "LocalEnum.EnemyKind", f"F_Enum({enemy_kind_uid})"),
        field_def("arena", enemy_arena_field_uid, "Int", "F_Int"),
        field_def("healthMultiplier", enemy_health_field_uid, "Float", "F_Float"),
        field_def("patrolRange", enemy_patrol_field_uid, "Float", "F_Float"),
    ]
    gate_fields = [field_def("arena", gate_arena_field_uid, "Int", "F_Int")]
    backdrop_fields = [
        field_def("kind", backdrop_kind_field_uid, "LocalEnum.BackdropKind", f"F_Enum({backdrop_kind_uid})"),
        field_def("scale", backdrop_scale_field_uid, "Float", "F_Float"),
        field_def("depth", backdrop_depth_field_uid, "Int", "F_Int"),
    ]

    project_iid = iid("project:windheart-sky-city")
    level_iid = iid("level:windheart-sky-city")
    return {
        "__header__": {
            "fileType": "LDtk Project JSON",
            "app": "LDtk",
            "doc": "https://ldtk.io/json",
            "schema": "https://ldtk.io/files/JSON_SCHEMA.json",
            "appAuthor": "Sebastien 'deepnight' Benard",
            "appVersion": "1.5.3",
            "url": "https://ldtk.io",
        },
        "jsonVersion": "1.5.3",
        "appBuildId": 473738,
        "nextUid": 600,
        "identifierStyle": "Capitalize",
        "iid": project_iid,
        "worldLayout": "Free",
        "worldGridWidth": WIDTH * GRID,
        "worldGridHeight": HEIGHT * GRID,
        "defaultLevelWidth": WIDTH * GRID,
        "defaultLevelHeight": HEIGHT * GRID,
        "defaultPivotX": 0,
        "defaultPivotY": 0,
        "defaultGridSize": GRID,
        "defaultEntityWidth": GRID,
        "defaultEntityHeight": GRID,
        "bgColor": "#75BDE7",
        "defaultLevelBgColor": "#75BDE7",
        "minifyJson": False,
        "externalLevels": False,
        "exportTiled": False,
        "simplifiedExport": False,
        "imageExportMode": "None",
        "exportLevelBg": True,
        "pngFilePattern": None,
        "backupOnSave": False,
        "backupLimit": 10,
        "backupRelPath": None,
        "levelNamePattern": "Level_%idx",
        "tutorialDesc": None,
        "flags": [],
        "toc": [],
        "customCommands": [],
        "dummyWorldIid": iid("world:default"),
        "worlds": [],
        "defs": {
            "layers": [gameplay_def, decor_def, terrain_def, collision_def],
            "entities": [
                entity_def("PlayerStart", 200, "#4FE6FF", max_count=1),
                entity_def("Checkpoint", 201, "#FFD166", [field_def("id", checkpoint_id_uid, "Int", "F_Int")]),
                entity_def("EnemySpawn", 202, "#E85D75", enemy_fields),
                entity_def("CombatGate", 203, "#58E6FF", gate_fields, GRID, 288, False, True),
                entity_def("Goal", 204, "#8FFFFF", max_count=1),
                entity_def("Backdrop", 205, "#D8F3FF", backdrop_fields),
            ],
            "tilesets": [
                {
                    "__cWid": 8,
                    "__cHei": 1,
                    "identifier": "SkyCityTiles",
                    "uid": TILESET_UID,
                    "relPath": TILESET_PATH,
                    "embedAtlas": None,
                    "pxWid": 256,
                    "pxHei": 32,
                    "tileGridSize": GRID,
                    "spacing": 0,
                    "padding": 0,
                    "tags": [],
                    "tagsSourceEnumUid": None,
                    "enumTags": [],
                    "customData": [],
                    "savedSelections": [],
                    "cachedPixelData": None,
                }
            ],
            "enums": [
                {
                    "identifier": "EnemyKind",
                    "uid": enemy_kind_uid,
                    "values": [
                        {"id": "Slime", "tileRect": None, "color": 7919241},
                        {"id": "Familiar", "tileRect": None, "color": 11051263},
                        {"id": "HeroicSpirit", "tileRect": None, "color": 15228277},
                    ],
                    "iconTilesetUid": None,
                    "externalRelPath": None,
                    "externalFileChecksum": None,
                    "tags": [],
                },
                {
                    "identifier": "BackdropKind",
                    "uid": backdrop_kind_uid,
                    "values": [
                        {"id": name, "tileRect": None, "color": color}
                        for name, color in [
                            ("Cloud", 14218239), ("Island", 8814474), ("Tower", 10724780),
                            ("Windmill", 12034385), ("Aqueduct", 11711192), ("Tree", 4356927),
                            ("Waterfall", 6089983), ("Temple", 13748195), ("Crystal", 6619135),
                        ]
                    ],
                    "iconTilesetUid": None,
                    "externalRelPath": None,
                    "externalFileChecksum": None,
                    "tags": [],
                },
            ],
            "externalEnums": [],
            "levelFields": [],
        },
        "levels": [
            {
                "identifier": "Windheart_Sky_City",
                "iid": level_iid,
                "uid": LEVEL_UID,
                "worldX": 0,
                "worldY": 0,
                "worldDepth": 0,
                "pxWid": WIDTH * GRID,
                "pxHei": HEIGHT * GRID,
                "__bgColor": "#75BDE7",
                "bgColor": None,
                "useAutoIdentifier": False,
                "bgRelPath": None,
                "bgPos": None,
                "bgPivotX": 0.5,
                "bgPivotY": 0.5,
                "__smartColor": "#A7D8ED",
                "__bgPos": None,
                "externalRelPath": None,
                "fieldInstances": [],
                "layerInstances": [gameplay, decor, terrain, collision_layer],
                "__neighbours": [],
            }
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail when the committed LDtk project differs from deterministic output",
    )
    args = parser.parse_args()
    rendered = json.dumps(build_project(), ensure_ascii=False, indent=2) + "\n"
    if args.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != rendered:
            raise SystemExit(f"generated LDtk project is stale: {OUTPUT}")
        print(f"up to date: {OUTPUT}")
        return

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(rendered, encoding="utf-8")
    print(OUTPUT)


if __name__ == "__main__":
    main()
