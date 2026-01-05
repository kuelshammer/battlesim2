/**
 * CrosshairContext - Shared hover state for synchronized crosshair across all Skyline charts
 *
 * Syncs across:
 * - HP Skyline (4 characters)
 * - Resource Skyline (4 characters)
 * - Death Bar (4 characters)
 *
 * Total: 8 charts synchronized by bucket position
 */

import React, { createContext, useContext, useState, useCallback, ReactNode, useMemo, memo } from 'react';
import { PercentileBucket, CharacterBucketData } from '@/model/skylineTypes';

export interface CrosshairState {
    /** Currently hovered bucket (1-100) or null */
    bucketIndex: number | null;
    /** X-coordinate relative to each chart (for crosshair line position) */
    xPosition: number | null;
    /** Currently hovered character ID or null */
    hoveredCharacterId: string | null;
    /** Data for all characters at this bucket */
    bucketData: {
        bucket: PercentileBucket | null;
        characters: {
            [characterId: string]: {
                hp: CharacterBucketData | null;
                resources: CharacterBucketData | null;
                name: string;
            };
        };
    } | null;
}

export interface CrosshairContextValue {
    /** Current crosshair state */
    state: CrosshairState;
    /** Set crosshair position (called from any chart) */
    setCrosshair: (bucketIndex: number | null, xPosition: number | null) => void;
    /** Set hovered character */
    setHoveredCharacter: (characterId: string | null) => void;
    /** Clear crosshair */
    clearCrosshair: () => void;
    /** Register a bucket data source (for crosshair tooltips) */
    registerBuckets: (sourceId: string, buckets: PercentileBucket[]) => void;
    /** Unregister a bucket data source */
    unregisterBuckets: (sourceId: string) => void;
}

const CrosshairContext = createContext<CrosshairContextValue | undefined>(undefined);

export interface CrosshairProviderProps {
    children: ReactNode;
}

/**
 * Crosshair Provider for synchronized hover state across all Skyline components
 */
export const CrosshairProvider: React.FC<CrosshairProviderProps> = memo(({
    children,
}) => {
    const [bucketIndex, setBucketIndex] = useState<number | null>(null);
    const [xPosition, setXPosition] = useState<number | null>(null);
    const [hoveredCharacterId, setHoveredCharacterId] = useState<string | null>(null);
    const [bucketSources, setBucketSources] = useState<Map<string, PercentileBucket[]>>(new Map());

    // Get combined bucket data from all sources
    const bucketData = useMemo(() => {
        if (bucketIndex === null || bucketIndex < 1) return null;

        // Get bucket from first source (HP or Resource - both have same buckets)
        const firstSource = Array.from(bucketSources.values())[0];
        if (!firstSource || bucketIndex - 1 >= firstSource.length) return null;

        const bucket = firstSource[bucketIndex - 1];

        // Aggregate character data from all sources
        const characters: { [charId: string]: { hp: CharacterBucketData | null; resources: CharacterBucketData | null; name: string } } = {};

        bucket.characters.forEach(char => {
            characters[char.id] = {
                hp: char,
                resources: char,
                name: char.name,
            };
        });

        return { bucket, characters };
    }, [bucketIndex, bucketSources]);

    const setCrosshair = useCallback((newBucketIndex: number | null, newXPosition: number | null) => {
        setBucketIndex(newBucketIndex);
        setXPosition(newXPosition);
    }, []);

    const setHoveredCharacter = useCallback((characterId: string | null) => {
        setHoveredCharacterId(characterId);
    }, []);

    const clearCrosshair = useCallback(() => {
        setBucketIndex(null);
        setXPosition(null);
        setHoveredCharacterId(null);
    }, []);

    const registerBuckets = useCallback((sourceId: string, buckets: PercentileBucket[]) => {
        setBucketSources(prev => new Map(prev).set(sourceId, buckets));
    }, []);

    const unregisterBuckets = useCallback((sourceId: string) => {
        setBucketSources(prev => {
            const next = new Map(prev);
            next.delete(sourceId);
            return next;
        });
    }, []);

    const value: CrosshairContextValue = {
        state: {
            bucketIndex,
            xPosition,
            hoveredCharacterId,
            bucketData,
        },
        setCrosshair,
        setHoveredCharacter,
        clearCrosshair,
        registerBuckets,
        unregisterBuckets,
    };

    return (
        <CrosshairContext.Provider value={value}>
            {children}
        </CrosshairContext.Provider>
    );
});

CrosshairProvider.displayName = 'CrosshairProvider';

/**
 * Hook to use crosshair context
 */
export function useCrosshair(): CrosshairContextValue {
    const context = useContext(CrosshairContext);
    if (!context) {
        throw new Error('useCrosshair must be used within CrosshairProvider');
    }
    return context;
}

/**
 * Hook to register bucket data and auto-cleanup
 */
export function useCrosshairBucketRegistration(sourceId: string, buckets: PercentileBucket[] | null) {
    const { registerBuckets, unregisterBuckets } = useCrosshair();

    React.useEffect(() => {
        if (buckets) {
            registerBuckets(sourceId, buckets);
        }

        return () => {
            unregisterBuckets(sourceId);
        };
    }, [sourceId, buckets, registerBuckets, unregisterBuckets]);
}
