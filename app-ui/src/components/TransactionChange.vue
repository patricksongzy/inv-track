<template>
  <form>
    <div class="field">
      <label class="label">Item ID</label>
      <div class="control">
        <input
          class="input"
          type="number"
          placeholder="ID"
          v-model="itemId"
          required
        />
      </div>
      <p class="help">Required</p>
    </div>
    <div class="field">
      <label class="label">Location ID</label>
      <div class="control">
        <input
          class="input"
          type="number"
          placeholder="ID"
          v-model="locationId"
        />
      </div>
    </div>
    <div class="field">
      <label class="label">Transaction Date</label>
      <div class="control">
        <input type="datetime-local" v-model="transactionDate" />
      </div>
    </div>
    <div class="field">
      <label class="label">Quantity</label>
      <div class="control">
        <input
          class="input"
          type="number"
          placeholder="Quantity"
          v-model="quantity"
          required
        />
      </div>
      <p class="help">Required</p>
    </div>
    <div class="field">
      <label class="label">Comment</label>
      <div class="control">
        <input
          class="input"
          type="text"
          placeholder="Comment"
          v-model="comment"
        />
      </div>
    </div>
    <div class="field is-grouped">
      <div class="control">
        <input
          class="button is-primary"
          type="submit"
          value="Submit"
          v-on:click="submitTransaction"
        />
      </div>
      <div class="control">
        <input
          v-if="this.$route.params.id"
          class="button"
          type="button"
          value="Cancel"
          v-on:click="$emit('submit')"
        />
      </div>
    </div>
  </form>
</template>
<script>
import { useRoute } from 'vue-router';
import { useMutation } from '@urql/vue';

export default {
  setup() {
    const route = useRoute();
    const [actionString, variableString, variableAssignment] = route.params.id
      ? ['update', '$id: TransactionId!, ', 'id: $id, ']
      : ['create', '', ''];
    const changeTransaction = useMutation(`
      mutation (${variableString}$transaction: InsertableTransaction!) {
        ${actionString}Transaction(${variableAssignment}transaction: $transaction) {
          id
        }
      }
    `);

    return {
      changeTransaction(
        itemId,
        locationId,
        transactionDate,
        quantity,
        comment
      ) {
        let variables = {
          transaction: Object.assign(
            {
              itemId: itemId,
              locationId: locationId,
              quantity: quantity,
            },
            transactionDate ? { transactionDate: transactionDate } : null,
            comment ? { comment: comment } : null
          ),
        };
        Object.assign(
          variables,
          route.params.id ? { id: parseInt(route.params.id) } : null
        );
        changeTransaction.executeMutation(variables).then((result) => {
          if (result.error) {
            console.log(result.error.graphQLErrors);
            alert(JSON.stringify(result.error.graphQLErrors[0]));
          } else {
            this.$emit('submit');
            alert('Changed!');
          }
        });
      },
    };
  },
  props: [
    'initialItemId',
    'initialLocationId',
    'initialTransactionDate',
    'initialQuantity',
    'initialComment',
  ],
  data: function () {
    return {
      itemId: this.initialItemId,
      locationId: this.initialLocationId,
      transactionDate: this.initialTransactionDate,
      quantity: this.initialQuantity,
      comment: this.initialComment,
    };
  },
  methods: {
    submitTransaction: function (e) {
      e.preventDefault();
      let dateString = null;
      if (this.transactionDate) {
        dateString = new Date(this.transactionDate).toISOString();
      }
      this.changeTransaction(
        this.itemId,
        this.locationId,
        dateString,
        this.quantity,
        this.comment
      );
    },
  },
};
</script>