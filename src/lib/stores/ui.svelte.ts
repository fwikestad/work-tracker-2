let showQuickAdd = $state(false);
let showSearch = $state(false);
let searchQuery = $state('');

export const uiStore = {
  get quickAdd() {
    return showQuickAdd;
  },
  get search() {
    return showSearch;
  },
  get query() {
    return searchQuery;
  },

  openQuickAdd() {
    showQuickAdd = true;
    showSearch = false;
  },
  closeQuickAdd() {
    showQuickAdd = false;
  },

  openSearch() {
    showSearch = true;
    showQuickAdd = false;
  },
  closeSearch() {
    showSearch = false;
    searchQuery = '';
  },

  setQuery(q: string) {
    searchQuery = q;
  }
};
