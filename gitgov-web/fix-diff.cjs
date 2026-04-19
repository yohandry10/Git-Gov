// This script reconstructs the Hero.tsx file by stripping diff markers
// The file was accidentally saved as a unified diff patch instead of clean code
const fs = require('fs');
const content = fs.readFileSync('components/marketing/Hero.tsx', 'utf8');
const lines = content.split(/\r?\n/);

// Extract clean lines from the diff format
// Lines starting with '+' (but not '+++') are new content to keep
// Lines starting with ' ' (space) are context (unchanged) to keep
// Lines starting with '-' (but not '---') are removed content - skip
// Lines starting with '@@' are hunk headers - skip
// Lines starting with '---' or '+++' are file headers - skip
const cleanLines = [];
let inDiff = false;

for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    
    // Detect diff headers
    if (line.startsWith('--- ') || line.startsWith('+++ ')) {
        inDiff = true;
        continue;
    }
    
    if (line.startsWith('@@')) {
        continue;
    }
    
    if (inDiff) {
        if (line.startsWith('-')) {
            // Old content, skip
            continue;
        } else if (line.startsWith('+')) {
            // New content, keep (remove the leading '+')
            cleanLines.push(line.substring(1));
        } else if (line.startsWith(' ')) {
            // Context line, keep (remove leading space)
            cleanLines.push(line.substring(1));
        } else {
            // Not a diff line  
            cleanLines.push(line);
        }
    } else {
        cleanLines.push(line);
    }
}

console.log('Original lines:', lines.length);
console.log('Clean lines:', cleanLines.length);
console.log('First 5 clean lines:');
for (let i = 0; i < 5; i++) {
    console.log(i + 1, JSON.stringify(cleanLines[i]));
}

fs.writeFileSync('components/marketing/Hero.tsx', cleanLines.join('\r\n'));
console.log('File saved successfully.');
