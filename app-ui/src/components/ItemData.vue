<template>
  <div v-if="item">
    <div v-if="!isEdit">
      <h1 class="title">
        {{ item.name }}
        <button class="button mr-1" v-on:click="isEdit = true">Edit</button>
        <button class="button is-danger" v-on:click="deleteItem">Delete</button>
      </h1>
      <p><strong>ID</strong>: {{ item.id }}</p>
      <p><strong>SKU</strong>: {{ item.sku ?? '--' }}</p>
      <p><strong>Quantity</strong>: {{ item.quantity }}</p>
      <p><strong>Supplier</strong>: {{ item.supplier ?? '--' }}</p>
      <p><strong>Description</strong>: {{ item.description ?? '--' }}</p>
      <hr />
      <h1 class="subtitle">Transactions</h1>
      <div class="table-container" v-if="item.transactions.length > 0">
        <table class="table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Location ID</th>
              <th>Location Name</th>
              <th>Location Address</th>
              <th>Transaction Date</th>
              <th>Quantity</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="transaction in item.transactions" :key="transaction.id">
              <td>{{ transaction.id }}</td>
              <td>{{ transaction?.location?.id ?? '--' }}</td>
              <td>{{ transaction?.location?.name ?? '--' }}</td>
              <td>{{ transaction?.location?.address ?? '--' }}</td>
              <td>{{ transaction.transactionDate?.split('T')[0] ?? '--' }}</td>
              <td>{{ transaction.quantity }}</td>
            </tr>
          </tbody>
          <tfoot>
            <tr>
              <th />
              <th />
              <th />
              <th />
              <th>Total</th>
              <th>{{ item.quantity }}</th>
            </tr>
          </tfoot>
        </table>
      </div>
      <div v-else>
        <p>No transactions for item.</p>
      </div>
    </div>
    <div v-else>
      <item-change
        v-on:submit="isEdit = false"
        :initialItemName="item.name"
        :initialSku="item.sku"
        :initialSupplier="item.supplier"
        :initialDescription="item.description"
      />
    </div>
  </div>
  <div v-else>
    <p>Item not found.</p>
  </div>
</template>
<script>
import { useRoute } from 'vue-router';
import { useClientHandle } from '@urql/vue';
import ItemChange from './ItemChange.vue';

export default {
  components: { ItemChange },
  data: () => {
    return {
      isEdit: false,
    };
  },
  async setup() {
    const handle = useClientHandle();
    const route = useRoute();
    let id = route.params.id;

    // get the initial items
    let result = await handle
      .useQuery({
        query: `
      {
        item(id: ${id}) {
          id,
          sku,
          name,
          supplier,
          description,
          quantity,
          transactions {
            id,
            location {
              id,
              name,
              address,
            },
            transactionDate,
            quantity,
          },
        }
      }
      `,
      })
      .executeQuery();
    if (result.data?.value?.item) {
      result = JSON.parse(JSON.stringify(result.data?.value?.item));
      // update the items live
      const handleSubscription = (_, response) => {
        const event = response.itemSubscription;
        if (event.data.id == id) {
          if (event.modification === 'UPDATE') {
            return event.data;
          }
        }
      };

      // subscribe to item changes
      // note this is very inefficient
      const subscription = handle.useSubscription(
        {
          query: `
        subscription {
          itemSubscription {
            modification,
            data {
              id,
              sku,
              name,
              supplier,
              description,
              quantity,
              transactions {
                id,
                location {
                  id,
                  name,
                  address,
                },
                transactionDate,
                quantity,
              },
            }
          }
        }
        `,
        },
        handleSubscription
      );

      const deleteItem = handle.useMutation(`
        mutation ($id: ItemId!) {
          deleteItem(id: $id) {
            id
          }
        }
      `);

      // hack to get things working
      return {
        base: result,
        live: subscription.data,
        deleteItem() {
          deleteItem.executeMutation({ id: parseInt(id) }).then((result) => {
            if (result.error) {
              alert(JSON.stringify(result.error.graphQLErrors[0]));
            } else {
              alert('Deleted!');
              this.$router.push('/items');
            }
          });
        },
      };
    } else {
      return {
        base: undefined,
        live: undefined,
      };
    }
  },
  computed: {
    item: function () {
      return this.live ?? this.base;
    },
  },
};
</script>