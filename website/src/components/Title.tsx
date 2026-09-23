import { useEffect } from 'react';

/** Sets the tab title per route. Returns nothing. */
export function Title({ text }: { text: string }): null {
  useEffect(() => {
    document.title = `${text} — Kineti`;
  }, [text]);
  return null;
}
