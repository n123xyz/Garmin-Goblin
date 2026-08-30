import type { HandleClientError } from '@sveltejs/kit';

export const handleError: HandleClientError = ({ error, event, status, message }) => {
  const errStr = error instanceof Error ? `${error.name}: ${error.message}\n${error.stack}` : String(error);
  console.error('🔥 [SvelteKit Client Error]:', errStr, event, status, message);
  return {
    message: errStr || message || 'Unknown client error',
  };
};
