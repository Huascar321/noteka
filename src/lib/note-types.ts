export const noteTypes = [
  ['reference', 'Reference'], ['plan', 'Plan'], ['web_clip', 'Web Clip'],
  ['voice', 'Voice Note'], ['journal', 'Journal'], ['daily', 'Daily'],
  ['meeting', 'Meeting'], ['topic_research', 'Topic Research'],
  ['outlier_content', 'Outlier Content'], ['creator_tips', 'Creator Tips'],
  ['tools', 'Tools'], ['book', 'Book'], ['recipe', 'Recipe'],
  ['idea', 'Idea'], ['lecture', 'Lecture'],
] as const;

export const noteTypeLabels: Record<string, string> = Object.fromEntries(noteTypes);
