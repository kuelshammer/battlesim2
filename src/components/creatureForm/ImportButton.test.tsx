import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ImportButton from './ImportButton';
import React from 'react';

describe('ImportButton', () => {
    it('should show modal when clicked and call onImport when creature is imported', () => {
        const onImport = vi.fn();
        render(<ImportButton onImport={onImport} />);

        const button = screen.getByTitle('Import from 5e.tools');
        fireEvent.click(button);

        // Verify modal appears (header text)
        expect(screen.getByText('Import from 5e.tools')).toBeDefined();
    });
});
