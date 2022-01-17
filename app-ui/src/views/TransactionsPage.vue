<template>
  <h1 class="title">Transactions</h1>
  <Suspense>
    <template #default>
      <transactions-list />
    </template>
    <template #fallback>
      <p>Loading transactions.</p>
    </template>
  </Suspense>
  <hr />
  <h1 class="subtitle">Create Transaction</h1>
  <transaction-change />
</template>
<script>
import TransactionsList from '../components/TransactionsList.vue';
import TransactionChange from '../components/TransactionChange.vue';

export default {
  components: {
    TransactionsList,
    TransactionChange,
  },
};
</script>
