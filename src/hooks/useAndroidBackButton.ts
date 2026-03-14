import { useEffect, useCallback, useRef } from 'react';
import { onBackKeyDown } from 'tauri-plugin-app-events-api';

export function useAndroidBackButton(
  currentView: "idle" | "summary" | "detail",
  onBack: () => void
) {
  const onBackRef = useRef(onBack);
  onBackRef.current = onBack;
  
  const handleBack = useCallback(() => {
    const shouldIntercept = currentView === "detail" || currentView === "summary";
    
    if (shouldIntercept) {
      onBackRef.current();
      return false;
    }
    return true;
  }, [currentView]);

  useEffect(() => {
    onBackKeyDown(handleBack);

    return () => {
    };
  }, [handleBack]);

  return handleBack;
}
