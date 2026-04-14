let isWidgetMode = $state(false);

export const widgetStore = {
  get isWidgetMode() {
    return isWidgetMode;
  },
  setWidgetMode(value: boolean) {
    isWidgetMode = value;
  },
};
