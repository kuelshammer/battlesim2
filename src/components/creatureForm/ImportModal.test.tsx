import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ImportModal from './ImportModal';
import React from 'react';

describe('ImportModal', () => {
    it('should call onImport with mapped creature when valid JSON is pasted', () => {
        const onImport = vi.fn();
        const onCancel = vi.fn();
        render(<ImportModal onImport={onImport} onCancel={onCancel} />);

        const textarea = screen.getByPlaceholderText(/name/);
        const validJson = JSON.stringify({
            name: "Test Monster",
            hp: { average: 10 },
            ac: [15]
        });

        fireEvent.change(textarea, { target: { value: validJson } });
        
        const importBtn = screen.getByText('Import Creature');
        fireEvent.click(importBtn);

        expect(onImport).toHaveBeenCalled();
        const importedCreature = onImport.mock.calls[0][0];
        expect(importedCreature.name).toBe("Test Monster");
        expect(importedCreature.ac).toBe(15);
    });

    it('should handle dirty JSON input (like leading j artifact)', () => {
        const onImport = vi.fn();
        const onCancel = vi.fn();
        render(<ImportModal onImport={onImport} onCancel={onCancel} />);

        const textarea = screen.getByPlaceholderText(/name/);
        const dirtyJson = 'j' + JSON.stringify({
            name: "Dirty Monster",
            hp: { average: 5 },
            ac: [10]
        });

        fireEvent.change(textarea, { target: { value: dirtyJson } });
        
        const importBtn = screen.getByText('Import Creature');
        fireEvent.click(importBtn);

        expect(onImport).toHaveBeenCalled();
        const importedCreature = onImport.mock.calls[0][0];
        expect(importedCreature.name).toBe("Dirty Monster");
    });

    it('should show error message when invalid JSON is pasted', () => {
        const onImport = vi.fn();
        const onCancel = vi.fn();
        render(<ImportModal onImport={onImport} onCancel={onCancel} />);

        const textarea = screen.getByPlaceholderText(/name/);
        fireEvent.change(textarea, { target: { value: 'invalid json' } });
        
        const importBtn = screen.getByText('Import Creature');
        fireEvent.click(importBtn);

        expect(screen.getByText(/No valid JSON found/)).toBeDefined();
        expect(onImport).not.toHaveBeenCalled();
    });
});
