const fs = require('fs');

let content = fs.readFileSync('src/data/actions.ts', 'utf8');

// Find all remaining createTemplate calls that don't have the new fields yet
const lines = content.split('\n');
const result = [];
let i = 0;

while (i < lines.length) {
    const line = lines[i];
    
    if (line.includes('createTemplate({') && 
        !line.includes('Bane:') && 
        !line.includes('Bless:') && 
        !line.includes('Fireball:') && 
        !line.includes('Haste:')) {
        
        // Found a template we need to fix
        result.push(line); // createTemplate({ line
        
        // Look for actionSlot line and add our fields after it
        i++;
        let foundActionSlot = false;
        while (i < lines.length && !lines[i].includes('},')) {
            const currentLine = lines[i];
            
            if (currentLine.includes('actionSlot:')) {
                result.push(currentLine);
                // Add our new fields after actionSlot
                result.push('        cost: [], // TODO: Add proper costs');
                result.push('        requirements: [], // TODO: Add proper requirements');
                result.push('        tags: [], // TODO: Add proper tags');
                foundActionSlot = true;
            } else {
                result.push(currentLine);
            }
            i++;
        }
        
        if (!foundActionSlot) {
            // If no actionSlot found, add fields at the beginning
            result.push('        cost: [], // TODO: Add proper costs');
            result.push('        requirements: [], // TODO: Add proper requirements');
            result.push('        tags: [], // TODO: Add proper tags');
        }
        
        if (i < lines.length) {
            result.push(lines[i]); // the closing }, line
        }
    } else {
        result.push(line);
    }
    i++;
}

fs.writeFileSync('src/data/actions.ts', result.join('\n'));
console.log('Fixed action templates');
