import React from 'react';
import { useMap } from './mapVisualizationHooks';
import { ExpandableSection } from '../components/ExpandableSection';
import { LoadingState } from '../components/LoadingState';
import { EmptyState } from '../components/EmptyState';

const COLORS = {
  WALKABLE_TERRAIN: '#2a4a2a',
  UNWALKABLE_TERRAIN: '#4a4a4a',
  UNEXPLORED: '#000000',
  ALLIED_UNITS: '#0000FF',
  ENEMY_UNITS: '#FF0000',
  MINERALS: '#00FFFF',
  GAS: '#00FF00',
  UNIT_STROKE: '#FFFFFF',
};

interface MapData {
  width: number;
  height: number;
  explored: boolean[][];
  walkability: boolean[][];
  resources: Array<{
    x: number;
    y: number;
    resource_type: string;
  }>;
  units: Array<{
    x: number;
    y: number;
    is_ally: boolean;
  }>;
}

const Legend: React.FC = () => {
  const legendItems = [
    { color: COLORS.WALKABLE_TERRAIN, label: 'Walkable Terrain' },
    { color: COLORS.UNWALKABLE_TERRAIN, label: 'Unwalkable Terrain' },
    { color: COLORS.UNEXPLORED, label: 'Unexplored' },
    { color: COLORS.ALLIED_UNITS, label: 'Allied Units' },
    { color: COLORS.ENEMY_UNITS, label: 'Enemy Units' },
    { color: COLORS.MINERALS, label: 'Minerals' },
    { color: COLORS.GAS, label: 'Gas' },
  ];

  return (
    <div className="flex flex-wrap gap-4 mb-4 p-4 bg-bg-secondary border border-border-primary rounded">
      {legendItems.map((item) => (
        <div key={item.label} className="flex items-center gap-2">
          <div
            className="w-5 h-5 rounded border border-border-accent"
            style={{ background: item.color }}
          />
          <span className="text-sm text-text-primary">{item.label}</span>
        </div>
      ))}
    </div>
  );
};

const MapSvg: React.FC<{ mapData: MapData }> = ({ mapData }) => {
  if (!mapData || mapData.width === 0 || mapData.height === 0) {
    return <EmptyState message="Invalid map data" />;
  }

  const scale = 3; // pixels per walk tile
  const svgWidth = mapData.width * scale;
  const svgHeight = mapData.height * scale;

  return (
    <div className="bg-black rounded overflow-auto max-h-150 border border-border-primary">
      <svg
        width={svgWidth}
        height={svgHeight}
        viewBox={`0 0 ${svgWidth} ${svgHeight}`}
        xmlns="http://www.w3.org/2000/svg"
      >
        {/* Background (unexplored areas) */}
        <rect width={svgWidth} height={svgHeight} fill={COLORS.UNEXPLORED} />

        {/* Draw explored and walkability */}
        {mapData.walkability.map((row, y) =>
          row.map((isWalkable, x) => {
            const isExplored = mapData.explored[y]?.[x] || false;
            if (!isExplored) return null;
            
            const color = isWalkable ? COLORS.WALKABLE_TERRAIN : COLORS.UNWALKABLE_TERRAIN;
            return (
              <rect
                key={`${x}-${y}`}
                x={x * scale}
                y={y * scale}
                width={scale}
                height={scale}
                fill={color}
              />
            );
          })
        )}

        {/* Draw resources */}
        {mapData.resources?.map((resource, i) => {
          const color = resource.resource_type.includes('Geyser')
            ? COLORS.GAS
            : COLORS.MINERALS;
          const cx = resource.x * scale + scale / 2;
          const cy = resource.y * scale + scale / 2;
          
          return (
            <circle
              key={`resource-${i}`}
              cx={cx}
              cy={cy}
              r={scale * 2}
              fill={color}
              opacity={0.8}
            />
          );
        })}

        {/* Draw units */}
        {mapData.units?.map((unit, i) => {
          const color = unit.is_ally ? COLORS.ALLIED_UNITS : COLORS.ENEMY_UNITS;
          const cx = unit.x * scale + scale / 2;
          const cy = unit.y * scale + scale / 2;
          
          return (
            <circle
              key={`unit-${i}`}
              cx={cx}
              cy={cy}
              r={scale}
              fill={color}
              stroke={COLORS.UNIT_STROKE}
              strokeWidth={0.5}
            />
          );
        })}
      </svg>
    </div>
  );
};

export const MapVisualization: React.FC = () => {
  const { data: mapData, isLoading, error } = useMap();

  const renderContent = () => {
    if (isLoading) {
      return <LoadingState message="Waiting for map data..." />;
    }

    if (error) {
      return <EmptyState message={`Error: ${error.message}`} />;
    }

    if (!mapData) {
      return <EmptyState message="No map data available" />;
    }

    return (
      <>
        <Legend />
        <MapSvg mapData={mapData.map_data} />
      </>
    );
  };

  return (
    <ExpandableSection title="Map Visualization" defaultExpanded={false}>
      {renderContent()}
    </ExpandableSection>
  );
};
