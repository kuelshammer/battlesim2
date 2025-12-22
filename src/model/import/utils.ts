export function cleanJsonInput(input: string): string {
    let cleaned = input.trim();

    // 1. Handle markdown code blocks
    const codeBlockRegex = /```(?:json)?\s*([\s\S]*?)\s*```/;
    const codeBlockMatch = cleaned.match(codeBlockRegex);
    if (codeBlockMatch) {
        cleaned = codeBlockMatch[1].trim();
    }

    // 2. Robust extraction: Find the outer-most { } or [ ]
    return extractJsonObject(cleaned);
}

function extractJsonObject(str: string): string {
    const firstBrace = str.indexOf('{');
    const firstBracket = str.indexOf('[');
    
    // Determine if we are looking for an object or an array
    const isArray = firstBracket !== -1 && (firstBrace === -1 || firstBracket < firstBrace);
    const startChar = isArray ? '[' : '{';
    const endChar = isArray ? ']' : '}';
    const startIndex = isArray ? firstBracket : firstBrace;

    if (startIndex === -1) return "";

    // Find the matching closing character by tracking depth
    let depth = 0;
    for (let i = startIndex; i < str.length; i++) {
        if (str[i] === startChar) depth++;
        else if (str[i] === endChar) depth--;

        if (depth === 0) {
            return str.substring(startIndex, i + 1);
        }
    }

    // Fallback to simple lastIndexOf if depth tracking fails (e.g. malformed but maybe parseable)
    const lastIndex = str.lastIndexOf(endChar);
    if (lastIndex > startIndex) {
        return str.substring(startIndex, lastIndex + 1);
    }

    return "";
}
