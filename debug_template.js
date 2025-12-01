// Debug script to test template action serialization
const { getFinalAction } = require('./src/data/actions')
const { ActionTemplates } = require('./src/data/actions')

// Test Bless template
const blessTemplate = ActionTemplates['Bless']
console.log('Original Bless template:', JSON.stringify(blessTemplate, null, 2))

// Create a template action like Acolyte uses
const templateAction = {
    id: 'test-bless',
    type: 'template',
    freq: 'at will',
    condition: 'not used yet',
    templateOptions: {
        templateName: 'Bless',
        target: 'ally with the highest DPR'
    }
}

const finalAction = getFinalAction(templateAction)
console.log('Final action after getFinalAction:', JSON.stringify(finalAction, null, 2))

// Check specifically if buff.displayName is preserved
if (finalAction.buff && finalAction.buff.displayName) {
    console.log('✅ displayName preserved:', finalAction.buff.displayName)
} else {
    console.log('❌ displayName missing:', finalAction.buff)
}