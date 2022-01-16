<template>
  <h1 class="title">Inventory Tracking</h1>
  <p>Navigate to any of the pages in the navbar to begin.</p>
  <p>
    Or visit
    <a v-bind:href="`http://${api_url}/playground`">the GraphQL playground</a>.
  </p>
</template>
<script>
export default {
  data() {
    return {
      api_url: process.env.VUE_APP_API_ADDRESS,
    };
  },
};
</script>