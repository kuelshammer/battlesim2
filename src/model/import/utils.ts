export function cleanJsonInput(input: string): string {
    let cleaned = input.trim();

    // 1. Handle markdown code blocks
    // Matches ```json { ... } ``` or ``` { ... } ```
    const codeBlockRegex = /```(?:json)?\s*([\s\S]*?)\s*```/;
    const codeBlockMatch = cleaned.match(codeBlockRegex);
    if (codeBlockMatch) {
        cleaned = codeBlockMatch[1].trim();
    }

    // 2. Extract the actual JSON object/array
    // This handles leading characters (like 'j') or trailing garbage
    // It finds the first '{' or '[' and the last '}' or ']'
    const firstBrace = cleaned.indexOf('{');
    const firstBracket = cleaned.indexOf('[');
    
    let start = -1;
    let end = -1;

    if (firstBrace !== -1 && (firstBracket === -1 || firstBrace < firstBracket)) {
        start = firstBrace;
        end = cleaned.lastIndexOf('}');
    } else if (firstBracket !== -1) {
        start = firstBracket;
        end = cleaned.lastIndexOf(']');
    }

    if (start !== -1 && end !== -1 && end > start) {
        return cleaned.substring(start, end + 1);
    }

    // If no valid start/end found but the original trim might be valid JSON (though unlikely if previous logic failed)
    return (cleaned.startsWith('{') || cleaned.startsWith('[')) ? cleaned : "";
}