What the Feature Does:
Problem Solved: Currently, the GUI shows aggregated stats from 201 runs, but the console log is from a random/first run - they don't match.

Solution:

After running 1005 simulations and selecting a quintile (based on luck slider)
Find the most representative run within that quintile using a similarity score
Generate a detailed, well-formatted combat log
Save it as a .txt file in GEMINI_REPORTS/combat_logs/
Display it in a GUI modal via a "Details" button
Key Benefits:

Logs now match the displayed stats
Better formatted for debugging
Accessible both in GUI and as downloadable text file
The plan includes:

Similarity scoring algorithm to find the best match
Enhanced log formatting (like your example with sections)
File system integration
React modal component design
Implementation phases (4-7 hours total work)