import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import AdjustmentPreview from './AdjustmentPreview';
import React from 'react';

describe('AdjustmentPreview Component', () => {
    const mockOriginalMonsters = [
        { id: 'm1', name: 'Goblin', hp: 7, ac: 15, saveBonus: 0, actions: [] }
    ] as unknown[];

    const mockAdjustmentResult = {
        monsters: [
            { id: 'm1', name: 'Goblin', hp: 12.4, ac: 16, saveBonus: 1, actions: [] }
        ],
        analysis: {
            scenarioName: 'Optimized',
            totalRuns: 100,
            deciles: [],
            battleDurationRounds: 3
        }
    } as unknown;

    it('should display original and optimized stats', () => {
        render(
            <AdjustmentPreview
                originalMonsters={mockOriginalMonsters}
                adjustmentResult={mockAdjustmentResult}
                onApply={vi.fn()}
                onCancel={vi.fn()}
            />
        );

        expect(screen.getByText('Goblin')).toBeDefined();
        expect(screen.getByText('7')).toBeDefined(); // Original HP
        expect(screen.getByText('12')).toBeDefined(); // Optimized HP (rounded)
        expect(screen.getByText('15')).toBeDefined(); // Original AC
        expect(screen.getByText('16')).toBeDefined(); // Optimized AC
    });

    it('should call onApply when apply button is clicked', () => {
        const onApply = vi.fn();
        render(
            <AdjustmentPreview
                originalMonsters={mockOriginalMonsters}
                adjustmentResult={mockAdjustmentResult}
                onApply={onApply}
                onCancel={vi.fn()}
            />
        );

        fireEvent.click(screen.getByText(/Apply Changes/i));
        expect(onApply).toHaveBeenCalled();
    });

    it('should call onCancel when cancel button is clicked', () => {
        const onCancel = vi.fn();
        render(
            <AdjustmentPreview
                originalMonsters={mockOriginalMonsters}
                adjustmentResult={mockAdjustmentResult}
                onApply={vi.fn()}
                onCancel={onCancel}
            />
        );

        fireEvent.click(screen.getByText(/Cancel/i));
        expect(onCancel).toHaveBeenCalled();
    });
});
