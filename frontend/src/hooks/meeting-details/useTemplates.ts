import { useState, useEffect, useCallback } from 'react';
import { invoke as invokeTauri } from '@tauri-apps/api/core';
import { toast } from 'sonner';
import Analytics from '@/lib/analytics';

export function useTemplates(
  meetingId: string,
  defaultTemplateId: string = 'standard_meeting'
) {
  const [availableTemplates, setAvailableTemplates] = useState<Array<{
    id: string;
    name: string;
    description: string;
  }>>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<string>(defaultTemplateId);

  // The backend is the source of truth for a saved workflow. Reset only when
  // navigating to a different session or when its persisted template changes.
  useEffect(() => {
    setSelectedTemplate(defaultTemplateId);
  }, [meetingId, defaultTemplateId]);

  // Fetch available templates on mount
  useEffect(() => {
    const fetchTemplates = async () => {
      try {
        const templates = await invokeTauri('api_list_templates') as Array<{
          id: string;
          name: string;
          description: string;
        }>;
        console.log('Available templates:', templates);
        setAvailableTemplates(templates);
      } catch (error) {
        console.error('Failed to fetch templates:', error);
      }
    };
    fetchTemplates();
  }, []);

  // Persist the workflow immediately so a reload, later summary generation, or
  // export uses the same template. Roll back the optimistic selection on error.
  const handleTemplateSelection = useCallback(async (templateId: string, templateName: string) => {
    if (templateId === selectedTemplate) return;

    const previousTemplateId = selectedTemplate;
    setSelectedTemplate(templateId);

    try {
      const saved = await invokeTauri('api_save_session_template', {
        meetingId,
        templateId,
      }) as { template_id: string; name: string };
      setSelectedTemplate(saved.template_id);
      toast.success('Template saved for this session', {
        description: `Using "${saved.name || templateName}" for future summary generation`,
      });
      Analytics.trackFeatureUsed('template_selected');
    } catch (error) {
      console.error('Failed to save session template:', error);
      setSelectedTemplate(previousTemplateId);
      toast.error('Could not save template selection', {
        description: 'Your previous session template has been restored.',
      });
    }
  }, [meetingId, selectedTemplate]);

  return {
    availableTemplates,
    selectedTemplate,
    handleTemplateSelection,
  };
}
