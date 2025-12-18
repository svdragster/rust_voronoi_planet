//! Color mapping for terrain types

use crate::terrain::BasicTerrainType;

/// RGBA color type
pub type TerrainColor = [f32; 4];

/// Trait for mapping terrain types to colors
pub trait ColorMapper<T> {
    /// Map a terrain type to an RGBA color
    fn map_color(&self, terrain: &T) -> TerrainColor;
}

/// Default color mapper for BasicTerrainType
#[derive(Debug, Clone, Copy, Default)]
pub struct BasicColorMapper;

impl ColorMapper<BasicTerrainType> for BasicColorMapper {
    fn map_color(&self, terrain: &BasicTerrainType) -> TerrainColor {
        match terrain {
            BasicTerrainType::Ocean => [0.1, 0.3, 0.7, 1.0],    // Deep blue
            BasicTerrainType::Beach => [0.9, 0.8, 0.5, 1.0],    // Sandy yellow
            BasicTerrainType::Land => [0.2, 0.6, 0.2, 1.0],     // Green
            BasicTerrainType::Mountain => [0.5, 0.5, 0.5, 1.0], // Gray
            BasicTerrainType::Ice => [0.95, 0.95, 1.0, 1.0],    // White-blue
        }
    }
}

/// Custom color mapper that allows setting colors for each terrain type
#[derive(Debug, Clone)]
pub struct CustomColorMapper {
    pub ocean: TerrainColor,
    pub beach: TerrainColor,
    pub land: TerrainColor,
    pub mountain: TerrainColor,
    pub ice: TerrainColor,
}

impl Default for CustomColorMapper {
    fn default() -> Self {
        Self {
            ocean: [0.1, 0.3, 0.7, 1.0],
            beach: [0.9, 0.8, 0.5, 1.0],
            land: [0.2, 0.6, 0.2, 1.0],
            mountain: [0.5, 0.5, 0.5, 1.0],
            ice: [0.95, 0.95, 1.0, 1.0],
        }
    }
}

impl ColorMapper<BasicTerrainType> for CustomColorMapper {
    fn map_color(&self, terrain: &BasicTerrainType) -> TerrainColor {
        match terrain {
            BasicTerrainType::Ocean => self.ocean,
            BasicTerrainType::Beach => self.beach,
            BasicTerrainType::Land => self.land,
            BasicTerrainType::Mountain => self.mountain,
            BasicTerrainType::Ice => self.ice,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_color_mapper() {
        let mapper = BasicColorMapper;

        let ocean_color = mapper.map_color(&BasicTerrainType::Ocean);
        assert_eq!(ocean_color[3], 1.0); // Alpha should be 1.0
        assert!(ocean_color[2] > 0.5); // Blue channel should be high

        let land_color = mapper.map_color(&BasicTerrainType::Land);
        assert_eq!(land_color[3], 1.0); // Alpha should be 1.0
        assert!(land_color[1] > 0.5); // Green channel should be high
    }

    #[test]
    fn test_custom_color_mapper() {
        let custom = CustomColorMapper {
            ocean: [0.0, 0.2, 0.5, 1.0],
            land: [0.3, 0.5, 0.1, 1.0],
            ..Default::default()
        };

        let ocean_color = custom.map_color(&BasicTerrainType::Ocean);
        assert_eq!(ocean_color, [0.0, 0.2, 0.5, 1.0]);

        let land_color = custom.map_color(&BasicTerrainType::Land);
        assert_eq!(land_color, [0.3, 0.5, 0.1, 1.0]);
    }

    #[test]
    fn test_all_terrain_types_have_colors() {
        let mapper = BasicColorMapper;

        // Ensure all terrain types have colors
        let _ = mapper.map_color(&BasicTerrainType::Ocean);
        let _ = mapper.map_color(&BasicTerrainType::Beach);
        let _ = mapper.map_color(&BasicTerrainType::Land);
        let _ = mapper.map_color(&BasicTerrainType::Mountain);
        let _ = mapper.map_color(&BasicTerrainType::Ice);
    }
}
